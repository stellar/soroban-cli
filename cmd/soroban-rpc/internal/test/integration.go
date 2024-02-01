package test

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"os/signal"
	"path"
	"path/filepath"
	"strconv"
	"sync"
	"syscall"
	"testing"
	"time"

	"github.com/sirupsen/logrus"
	"github.com/spf13/cobra"
	"github.com/stellar/go/clients/stellarcore"
	"github.com/stellar/go/keypair"
	"github.com/stellar/go/txnbuild"

	"github.com/stellar/soroban-tools/cmd/soroban-rpc/internal/config"
	"github.com/stellar/soroban-tools/cmd/soroban-rpc/internal/daemon"
	"github.com/stellar/soroban-tools/cmd/soroban-rpc/internal/db"
	"github.com/stellar/soroban-tools/cmd/soroban-rpc/internal/ledgerbucketwindow"
)

const (
	StandaloneNetworkPassphrase = "Standalone Network ; February 2017"
	stellarCoreProtocolVersion  = 20
	stellarCorePort             = 11626
	goModFile                   = "go.mod"
	goMonorepoGithubPath        = "github.com/stellar/go"
	friendbotURL                = "http://localhost:8000/friendbot"
	// Needed when Core is run with ARTIFICIALLY_ACCELERATE_TIME_FOR_TESTING=true
	checkpointFrequency    = 8
	sorobanRPCPort         = 8042
	adminPort              = 8080
	helloWorldContractPath = "../../../../target/wasm32-unknown-unknown/test-wasms/test_hello_world.wasm"
)

type Test struct {
	t *testing.T

	composePath string // docker compose yml file

	daemon *daemon.Daemon

	coreClient *stellarcore.Client

	masterAccount txnbuild.Account
	shutdownOnce  sync.Once
	shutdownCalls []func()
}

func NewTest(t *testing.T) *Test {
	if os.Getenv("SOROBAN_RPC_INTEGRATION_TESTS_ENABLED") == "" {
		t.Skip("skipping integration test: SOROBAN_RPC_INTEGRATION_TESTS_ENABLED not set")
	}
	coreBinaryPath := os.Getenv("SOROBAN_RPC_INTEGRATION_TESTS_CAPTIVE_CORE_BIN")
	if coreBinaryPath == "" {
		t.Fatal("missing SOROBAN_RPC_INTEGRATION_TESTS_CAPTIVE_CORE_BIN")
	}

	i := &Test{
		t:           t,
		composePath: findDockerComposePath(),
	}
	i.masterAccount = &txnbuild.SimpleAccount{
		AccountID: i.MasterKey().Address(),
		Sequence:  0,
	}
	i.runComposeCommand("up", "--detach", "--quiet-pull", "--no-color")
	i.prepareShutdownHandlers()
	i.coreClient = &stellarcore.Client{URL: "http://localhost:" + strconv.Itoa(stellarCorePort)}
	i.waitForCore()
	i.waitForCheckpoint()
	i.launchDaemon(coreBinaryPath)

	return i
}

func (i *Test) MasterKey() *keypair.Full {
	return keypair.Root(StandaloneNetworkPassphrase)
}

func (i *Test) MasterAccount() txnbuild.Account {
	return i.masterAccount
}

func (i *Test) sorobanRPCURL() string {
	return fmt.Sprintf("http://localhost:%d", sorobanRPCPort)
}

func (i *Test) adminURL() string {
	return fmt.Sprintf("http://localhost:%d", adminPort)
}

func (i *Test) waitForCheckpoint() {
	i.t.Log("Waiting for core to be up...")
	for t := 30 * time.Second; t >= 0; t -= time.Second {
		ctx, cancel := context.WithTimeout(context.Background(), time.Second)
		info, err := i.coreClient.Info(ctx)
		cancel()
		if err != nil {
			i.t.Logf("could not obtain info response: %v", err)
			time.Sleep(time.Second)
			continue
		}
		if info.Info.Ledger.Num <= checkpointFrequency {
			i.t.Logf("checkpoint not reached yet: %v", info)
			time.Sleep(time.Second)
			continue
		}
		return
	}
	i.t.Fatal("Core could not reach checkpoint ledger after 30s")
}

func (i *Test) launchDaemon(coreBinaryPath string) {
	var config config.Config
	cmd := &cobra.Command{}
	if err := config.AddFlags(cmd); err != nil {
		i.t.FailNow()
	}
	if err := config.SetValues(func(string) (string, bool) { return "", false }); err != nil {
		i.t.FailNow()
	}

	config.Endpoint = fmt.Sprintf("localhost:%d", sorobanRPCPort)
	config.AdminEndpoint = fmt.Sprintf("localhost:%d", adminPort)
	config.StellarCoreURL = "http://localhost:" + strconv.Itoa(stellarCorePort)
	config.CoreRequestTimeout = time.Second * 2
	config.StellarCoreBinaryPath = coreBinaryPath
	config.CaptiveCoreConfigPath = path.Join(i.composePath, "captive-core-integration-tests.cfg")
	config.CaptiveCoreStoragePath = i.t.TempDir()
	config.CaptiveCoreHTTPPort = 0
	config.FriendbotURL = friendbotURL
	config.NetworkPassphrase = StandaloneNetworkPassphrase
	config.HistoryArchiveURLs = []string{"http://localhost:1570"}
	config.LogLevel = logrus.DebugLevel
	config.SQLiteDBPath = path.Join(i.t.TempDir(), "soroban_rpc.sqlite")
	config.IngestionTimeout = 10 * time.Minute
	config.EventLedgerRetentionWindow = ledgerbucketwindow.DefaultEventLedgerRetentionWindow
	config.CheckpointFrequency = checkpointFrequency
	config.MaxHealthyLedgerLatency = time.Second * 10
	config.PreflightEnableDebug = true

	i.daemon = daemon.MustNew(&config)
	go i.daemon.Run()

	// wait for the storage to catch up for 1 minute
	info, err := i.coreClient.Info(context.Background())
	if err != nil {
		i.t.Fatalf("cannot obtain latest ledger from core: %v", err)
	}
	targetLedgerSequence := uint32(info.Info.Ledger.Num)

	reader := db.NewLedgerEntryReader(i.daemon.GetDB())
	success := false
	for t := 30; t >= 0; t -= 1 {
		sequence, err := reader.GetLatestLedgerSequence(context.Background())
		if err != nil {
			if err != db.ErrEmptyDB {
				i.t.Fatalf("cannot access ledger entry storage: %v", err)
			}
		} else {
			if sequence >= targetLedgerSequence {
				success = true
				break
			}
		}
		time.Sleep(time.Second)
	}
	if !success {
		i.t.Fatalf("LedgerEntryStorage failed to sync in 1 minute")
	}
}

// Runs a docker-compose command applied to the above configs
func (i *Test) runComposeCommand(args ...string) {
	integrationYaml := filepath.Join(i.composePath, "docker-compose.yml")

	cmdline := append([]string{"-f", integrationYaml}, args...)
	cmd := exec.Command("docker-compose", cmdline...)

	i.t.Log("Running", cmd.Env, cmd.Args)
	out, innerErr := cmd.Output()
	if exitErr, ok := innerErr.(*exec.ExitError); ok {
		fmt.Printf("stdout:\n%s\n", string(out))
		fmt.Printf("stderr:\n%s\n", string(exitErr.Stderr))
	}

	if innerErr != nil {
		i.t.Fatalf("Compose command failed: %v", innerErr)
	}
}

func (i *Test) prepareShutdownHandlers() {
	i.shutdownCalls = append(i.shutdownCalls,
		func() {
			if i.daemon != nil {
				i.daemon.Close()
			}
			i.runComposeCommand("down", "-v")
		},
	)

	// Register cleanup handlers (on panic and ctrl+c) so the containers are
	// stopped even if ingestion or testing fails.
	i.t.Cleanup(i.Shutdown)

	c := make(chan os.Signal, 1)
	signal.Notify(c, os.Interrupt, syscall.SIGTERM)
	go func() {
		<-c
		i.Shutdown()
		os.Exit(int(syscall.SIGTERM))
	}()
}

// Shutdown stops the integration tests and destroys all its associated
// resources. It will be implicitly called when the calling test (i.e. the
// `testing.Test` passed to `New()`) is finished if it hasn't been explicitly
// called before.
func (i *Test) Shutdown() {
	i.shutdownOnce.Do(func() {
		// run them in the opposite order in which they where added
		for callI := len(i.shutdownCalls) - 1; callI >= 0; callI-- {
			i.shutdownCalls[callI]()
		}
	})
}

// Wait for core to be up and manually close the first ledger
func (i *Test) waitForCore() {
	i.t.Log("Waiting for core to be up...")
	for t := 30 * time.Second; t >= 0; t -= time.Second {
		ctx, cancel := context.WithTimeout(context.Background(), time.Second)
		_, err := i.coreClient.Info(ctx)
		cancel()
		if err != nil {
			i.t.Logf("could not obtain info response: %v", err)
			time.Sleep(time.Second)
			continue
		}
		break
	}

	i.UpgradeProtocol(stellarCoreProtocolVersion)

	for t := 0; t < 5; t++ {
		ctx, cancel := context.WithTimeout(context.Background(), time.Second)
		info, err := i.coreClient.Info(ctx)
		cancel()
		if err != nil || !info.IsSynced() {
			i.t.Logf("Core is still not synced: %v %v", err, info)
			time.Sleep(time.Second)
			continue
		}
		i.t.Log("Core is up.")
		return
	}
	i.t.Fatal("Core could not sync after 30s")
}

// UpgradeProtocol arms Core with upgrade and blocks until protocol is upgraded.
func (i *Test) UpgradeProtocol(version uint32) {
	ctx, cancel := context.WithTimeout(context.Background(), time.Second)
	err := i.coreClient.Upgrade(ctx, int(version))
	cancel()
	if err != nil {
		i.t.Fatalf("could not upgrade protocol: %v", err)
	}

	for t := 0; t < 10; t++ {
		ctx, cancel := context.WithTimeout(context.Background(), time.Second)
		info, err := i.coreClient.Info(ctx)
		cancel()
		if err != nil {
			i.t.Logf("could not obtain info response: %v", err)
			time.Sleep(time.Second)
			continue
		}

		if info.Info.Ledger.Version == int(version) {
			i.t.Logf("Protocol upgraded to: %d", info.Info.Ledger.Version)
			return
		}
		time.Sleep(time.Second)
	}

	i.t.Fatalf("could not upgrade protocol in 10s")
}

// Cluttering code with if err != nil is absolute nonsense.
func panicIf(err error) {
	if err != nil {
		panic(err)
	}
}

// findProjectRoot iterates upward on the directory until go.mod file is found.
func findProjectRoot(current string) string {
	// Lets you check if a particular directory contains a file.
	directoryContainsFilename := func(dir string, filename string) bool {
		files, innerErr := os.ReadDir(dir)
		panicIf(innerErr)

		for _, file := range files {
			if file.Name() == filename {
				return true
			}
		}
		return false
	}
	var err error

	// In either case, we try to walk up the tree until we find "go.mod",
	// which we hope is the root directory of the project.
	for !directoryContainsFilename(current, goModFile) {
		current, err = filepath.Abs(filepath.Join(current, ".."))

		// FIXME: This only works on *nix-like systems.
		if err != nil || filepath.Base(current)[0] == filepath.Separator {
			fmt.Println("Failed to establish project root directory.")
			panic(err)
		}
	}
	return current
}

// findDockerComposePath performs a best-effort attempt to find the project's
// Docker Compose files.
func findDockerComposePath() string {
	current, err := os.Getwd()
	panicIf(err)

	//
	// We have a primary and backup attempt for finding the necessary docker
	// files: via $GOPATH and via local directory traversal.
	//

	if gopath := os.Getenv("GOPATH"); gopath != "" {
		monorepo := filepath.Join(gopath, "src", "github.com", "stellar", "soroban-tools")
		if _, err = os.Stat(monorepo); !os.IsNotExist(err) {
			current = monorepo
		}
	}

	current = findProjectRoot(current)

	// Directly jump down to the folder that should contain the configs
	return filepath.Join(current, "cmd", "soroban-rpc", "internal", "test")
}
