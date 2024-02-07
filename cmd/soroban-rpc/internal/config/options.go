package config

import (
	"fmt"
	"os"
	"os/exec"
	"reflect"
	"runtime"
	"time"

	"github.com/sirupsen/logrus"

	"github.com/stellar/go/network"
	"github.com/stellar/go/support/errors"
	"github.com/stellar/go/support/strutils"

	"github.com/stellar/soroban-tools/cmd/soroban-rpc/internal/ledgerbucketwindow"
)

const defaultHTTPEndpoint = "localhost:8000"

func (cfg *Config) options() ConfigOptions {
	if cfg.optionsCache != nil {
		return *cfg.optionsCache
	}
	defaultStellarCoreBinaryPath, _ := exec.LookPath("stellar-core")
	cfg.optionsCache = &ConfigOptions{
		{
			Name:      "config-path",
			EnvVar:    "SOROBAN_RPC_CONFIG_PATH",
			TomlKey:   "-",
			Usage:     "File path to the toml configuration file",
			ConfigKey: &cfg.ConfigPath,
		},
		{
			Name:         "config-strict",
			EnvVar:       "SOROBAN_RPC_CONFIG_STRICT",
			TomlKey:      "STRICT",
			Usage:        "Enable strict toml configuration file parsing. This will prevent unknown fields in the config toml from being parsed.",
			ConfigKey:    &cfg.Strict,
			DefaultValue: false,
		},
		{
			Name:         "endpoint",
			Usage:        "Endpoint to listen and serve on",
			ConfigKey:    &cfg.Endpoint,
			DefaultValue: defaultHTTPEndpoint,
		},
		{
			Name:      "admin-endpoint",
			Usage:     "Admin endpoint to listen and serve on. WARNING: this should not be accessible from the Internet and does not use TLS. \"\" (default) disables the admin server",
			ConfigKey: &cfg.AdminEndpoint,
		},
		{
			Name:      "stellar-core-url",
			Usage:     "URL used to query Stellar Core (local captive core by default)",
			ConfigKey: &cfg.StellarCoreURL,
			Validate: func(co *ConfigOption) error {
				// This is a bit awkward. We're actually setting a default, but we
				// can't do that until the config is fully parsed, so we do it as a
				// validator here.
				if cfg.StellarCoreURL == "" {
					cfg.StellarCoreURL = fmt.Sprintf("http://localhost:%d", cfg.CaptiveCoreHTTPPort)
				}
				return nil
			},
		},
		{
			Name:         "stellar-core-timeout",
			Usage:        "Timeout used when submitting requests to stellar-core",
			ConfigKey:    &cfg.CoreRequestTimeout,
			DefaultValue: 2 * time.Second,
		},
		{
			Name:         "stellar-captive-core-http-port",
			Usage:        "HTTP port for Captive Core to listen on (0 disables the HTTP server)",
			ConfigKey:    &cfg.CaptiveCoreHTTPPort,
			DefaultValue: uint(11626),
		},
		{
			Name:         "log-level",
			Usage:        "minimum log severity (debug, info, warn, error) to log",
			ConfigKey:    &cfg.LogLevel,
			DefaultValue: logrus.InfoLevel,
			CustomSetValue: func(option *ConfigOption, i interface{}) error {
				switch v := i.(type) {
				case nil:
					return nil
				case string:
					ll, err := logrus.ParseLevel(v)
					if err != nil {
						return fmt.Errorf("could not parse %s: %q", option.Name, v)
					}
					cfg.LogLevel = ll
				case logrus.Level:
					cfg.LogLevel = v
				case *logrus.Level:
					cfg.LogLevel = *v
				default:
					return fmt.Errorf("could not parse %s: %q", option.Name, v)
				}
				return nil
			},
			MarshalTOML: func(option *ConfigOption) (interface{}, error) {
				return cfg.LogLevel.String(), nil
			},
		},
		{
			Name:         "log-format",
			Usage:        "format used for output logs (json or text)",
			ConfigKey:    &cfg.LogFormat,
			DefaultValue: LogFormatText,
			CustomSetValue: func(option *ConfigOption, i interface{}) error {
				switch v := i.(type) {
				case nil:
					return nil
				case string:
					return errors.Wrapf(
						cfg.LogFormat.UnmarshalText([]byte(v)),
						"could not parse %s",
						option.Name,
					)
				case LogFormat:
					cfg.LogFormat = v
				case *LogFormat:
					cfg.LogFormat = *v
				default:
					return fmt.Errorf("could not parse %s: %q", option.Name, v)
				}
				return nil
			},
			MarshalTOML: func(option *ConfigOption) (interface{}, error) {
				return cfg.LogFormat.String()
			},
		},
		{
			Name:         "stellar-core-binary-path",
			Usage:        "path to stellar core binary",
			ConfigKey:    &cfg.StellarCoreBinaryPath,
			DefaultValue: defaultStellarCoreBinaryPath,
			Validate:     required,
		},
		{
			Name:      "captive-core-config-path",
			Usage:     "path to additional configuration for the Stellar Core configuration file used by captive core. It must, at least, include enough details to define a quorum set",
			ConfigKey: &cfg.CaptiveCoreConfigPath,
			Validate:  required,
		},
		{
			Name:      "captive-core-storage-path",
			Usage:     "Storage location for Captive Core bucket data",
			ConfigKey: &cfg.CaptiveCoreStoragePath,
			CustomSetValue: func(option *ConfigOption, i interface{}) error {
				switch v := i.(type) {
				case string:
					if v == "" || v == "." {
						cwd, err := os.Getwd()
						if err != nil {
							return fmt.Errorf("unable to determine the current directory: %s", err)
						}
						v = cwd
					}
					cfg.CaptiveCoreStoragePath = v
					return nil
				case nil:
					cwd, err := os.Getwd()
					if err != nil {
						return fmt.Errorf("unable to determine the current directory: %s", err)
					}
					cfg.CaptiveCoreStoragePath = cwd
					return nil
				default:
					return fmt.Errorf("could not parse %s: %v", option.Name, v)
				}
			},
		},
		{
			Name:      "history-archive-urls",
			Usage:     "comma-separated list of stellar history archives to connect with",
			ConfigKey: &cfg.HistoryArchiveURLs,
			Validate:  required,
		},
		{
			Name:      "friendbot-url",
			Usage:     "The friendbot URL to be returned by getNetwork endpoint",
			ConfigKey: &cfg.FriendbotURL,
		},
		{
			Name:      "network-passphrase",
			Usage:     "Network passphrase of the Stellar network transactions should be signed for. Commonly used values are \"" + network.FutureNetworkPassphrase + "\", \"" + network.TestNetworkPassphrase + "\" and \"" + network.PublicNetworkPassphrase + "\"",
			ConfigKey: &cfg.NetworkPassphrase,
			Validate:  required,
		},
		{
			Name:         "db-path",
			Usage:        "SQLite DB path",
			ConfigKey:    &cfg.SQLiteDBPath,
			DefaultValue: "soroban_rpc.sqlite",
		},
		{
			Name:         "ingestion-timeout",
			Usage:        "Ingestion Timeout when bootstrapping data (checkpoint and in-memory initialization) and preparing ledger reads",
			ConfigKey:    &cfg.IngestionTimeout,
			DefaultValue: 30 * time.Minute,
		},
		{
			Name:         "checkpoint-frequency",
			Usage:        "establishes how many ledgers exist between checkpoints, do NOT change this unless you really know what you are doing",
			ConfigKey:    &cfg.CheckpointFrequency,
			DefaultValue: uint32(64),
		},
		{
			Name: "event-retention-window",
			Usage: fmt.Sprintf("configures the event retention window expressed in number of ledgers,"+
				" the default value is %d which corresponds to about 24 hours of history", ledgerbucketwindow.DefaultEventLedgerRetentionWindow),
			ConfigKey:    &cfg.EventLedgerRetentionWindow,
			DefaultValue: uint32(ledgerbucketwindow.DefaultEventLedgerRetentionWindow),
			Validate:     positive,
		},
		{
			Name: "transaction-retention-window",
			Usage: "configures the transaction retention window expressed in number of ledgers," +
				" the default value is 1440 which corresponds to about 2 hours of history",
			ConfigKey:    &cfg.TransactionLedgerRetentionWindow,
			DefaultValue: uint32(1440),
			Validate:     positive,
		},

		{
			Name: "ledger-retention-window",
			Usage: "configures the common retention window for events and transactions expressed in number of ledgers," +
				" the default value is 17280 which corresponds to about 24 hours of history",
			ConfigKey:    &cfg.LedgerRetentionWindow,
			DefaultValue: uint32(ledgerbucketwindow.DefaultLedgerRetentionWindow),
			Validate:     positive,
		},

		{
			Name:         "max-events-limit",
			Usage:        "Maximum amount of events allowed in a single getEvents response",
			ConfigKey:    &cfg.MaxEventsLimit,
			DefaultValue: uint(10000),
		},
		{
			Name:         "default-events-limit",
			Usage:        "Default cap on the amount of events included in a single getEvents response",
			ConfigKey:    &cfg.DefaultEventsLimit,
			DefaultValue: uint(100),
			Validate: func(co *ConfigOption) error {
				if cfg.DefaultEventsLimit > cfg.MaxEventsLimit {
					return fmt.Errorf(
						"default-events-limit (%v) cannot exceed max-events-limit (%v)",
						cfg.DefaultEventsLimit,
						cfg.MaxEventsLimit,
					)
				}
				return nil
			},
		},
		{
			Name: "max-healthy-ledger-latency",
			Usage: "maximum ledger latency (i.e. time elapsed since the last known ledger closing time) considered to be healthy" +
				" (used for the /health endpoint)",
			ConfigKey:    &cfg.MaxHealthyLedgerLatency,
			DefaultValue: 30 * time.Second,
		},
		{
			Name:         "preflight-worker-count",
			Usage:        "Number of workers (read goroutines) used to compute preflights for the simulateTransaction endpoint. Defaults to the number of CPUs.",
			ConfigKey:    &cfg.PreflightWorkerCount,
			DefaultValue: uint(runtime.NumCPU()),
			Validate:     positive,
		},
		{
			Name:         "preflight-worker-queue-size",
			Usage:        "Maximum number of outstanding preflight requests for the simulateTransaction endpoint. Defaults to the number of CPUs.",
			ConfigKey:    &cfg.PreflightWorkerQueueSize,
			DefaultValue: uint(runtime.NumCPU()),
			Validate:     positive,
		},
		{
			Name:         "preflight-enable-debug",
			Usage:        "Enable debug information in preflighting (provides more detailed errors). It should not be enabled in production deployments.",
			ConfigKey:    &cfg.PreflightEnableDebug,
			DefaultValue: true,
		},
		{
			TomlKey:      strutils.KebabToConstantCase("request-backlog-global-queue-limit"),
			Usage:        "Maximum number of outstanding requests",
			ConfigKey:    &cfg.RequestBacklogGlobalQueueLimit,
			DefaultValue: uint(5000),
			Validate:     positive,
		},
		{
			TomlKey:      strutils.KebabToConstantCase("request-backlog-get-health-queue-limit"),
			Usage:        "Maximum number of outstanding GetHealth requests",
			ConfigKey:    &cfg.RequestBacklogGetHealthQueueLimit,
			DefaultValue: uint(1000),
			Validate:     positive,
		},
		{
			TomlKey:      strutils.KebabToConstantCase("request-backlog-get-events-queue-limit"),
			Usage:        "Maximum number of outstanding GetEvents requests",
			ConfigKey:    &cfg.RequestBacklogGetEventsQueueLimit,
			DefaultValue: uint(1000),
			Validate:     positive,
		},
		{
			TomlKey:      strutils.KebabToConstantCase("request-backlog-get-network-queue-limit"),
			Usage:        "Maximum number of outstanding GetNetwork requests",
			ConfigKey:    &cfg.RequestBacklogGetNetworkQueueLimit,
			DefaultValue: uint(1000),
			Validate:     positive,
		},
		{
			TomlKey:      strutils.KebabToConstantCase("request-backlog-get-latest-ledger-queue-limit"),
			Usage:        "Maximum number of outstanding GetLatestsLedger requests",
			ConfigKey:    &cfg.RequestBacklogGetLatestLedgerQueueLimit,
			DefaultValue: uint(1000),
			Validate:     positive,
		},
		{
			TomlKey:      strutils.KebabToConstantCase("request-backlog-get-ledger-entries-queue-limit"),
			Usage:        "Maximum number of outstanding GetLedgerEntries requests",
			ConfigKey:    &cfg.RequestBacklogGetLedgerEntriesQueueLimit,
			DefaultValue: uint(1000),
			Validate:     positive,
		},
		{
			TomlKey:      strutils.KebabToConstantCase("request-backlog-get-transaction-queue-limit"),
			Usage:        "Maximum number of outstanding GetTransaction requests",
			ConfigKey:    &cfg.RequestBacklogGetTransactionQueueLimit,
			DefaultValue: uint(1000),
			Validate:     positive,
		},
		{
			TomlKey:      strutils.KebabToConstantCase("request-backlog-send-transaction-queue-limit"),
			Usage:        "Maximum number of outstanding SendTransaction requests",
			ConfigKey:    &cfg.RequestBacklogSendTransactionQueueLimit,
			DefaultValue: uint(500),
			Validate:     positive,
		},
		{
			TomlKey:      strutils.KebabToConstantCase("request-backlog-simulate-transaction-queue-limit"),
			Usage:        "Maximum number of outstanding SimulateTransaction requests",
			ConfigKey:    &cfg.RequestBacklogSimulateTransactionQueueLimit,
			DefaultValue: uint(100),
			Validate:     positive,
		},
		{
			TomlKey:      strutils.KebabToConstantCase("request-execution-warning-threshold"),
			Usage:        "The request execution warning threshold is the predetermined maximum duration of time that a request can take to be processed before a warning would be generated",
			ConfigKey:    &cfg.RequestExecutionWarningThreshold,
			DefaultValue: 5 * time.Second,
		},
		{
			TomlKey:      strutils.KebabToConstantCase("max-request-execution-duration"),
			Usage:        "The max request execution duration is the predefined maximum duration of time allowed for processing a request. When that time elapses, the server would return 504 and abort the request's execution",
			ConfigKey:    &cfg.MaxRequestExecutionDuration,
			DefaultValue: 25 * time.Second,
		},
		{
			TomlKey:      strutils.KebabToConstantCase("max-get-health-execution-duration"),
			Usage:        "The maximum duration of time allowed for processing a getHealth request. When that time elapses, the rpc server would return -32001 and abort the request's execution",
			ConfigKey:    &cfg.MaxGetHealthExecutionDuration,
			DefaultValue: 5 * time.Second,
		},
		{
			TomlKey:      strutils.KebabToConstantCase("max-get_events-execution-duration"),
			Usage:        "The maximum duration of time allowed for processing a getEvents request. When that time elapses, the rpc server would return -32001 and abort the request's execution",
			ConfigKey:    &cfg.MaxGetEventsExecutionDuration,
			DefaultValue: 10 * time.Second,
		},
		{
			TomlKey:      strutils.KebabToConstantCase("max-get-network-execution-duration"),
			Usage:        "The maximum duration of time allowed for processing a getNetwork request. When that time elapses, the rpc server would return -32001 and abort the request's execution",
			ConfigKey:    &cfg.MaxGetNetworkExecutionDuration,
			DefaultValue: 5 * time.Second,
		},
		{
			TomlKey:      strutils.KebabToConstantCase("max-get-latest-ledger-execution-duration"),
			Usage:        "The maximum duration of time allowed for processing a getLatestLedger request. When that time elapses, the rpc server would return -32001 and abort the request's execution",
			ConfigKey:    &cfg.MaxGetLatestLedgerExecutionDuration,
			DefaultValue: 5 * time.Second,
		},
		{
			TomlKey:      strutils.KebabToConstantCase("max-get_ledger-entries-execution-duration"),
			Usage:        "The maximum duration of time allowed for processing a getLedgerEntries request. When that time elapses, the rpc server would return -32001 and abort the request's execution",
			ConfigKey:    &cfg.MaxGetLedgerEntriesExecutionDuration,
			DefaultValue: 5 * time.Second,
		},
		{
			TomlKey:      strutils.KebabToConstantCase("max-get-transaction-execution-duration"),
			Usage:        "The maximum duration of time allowed for processing a getTransaction request. When that time elapses, the rpc server would return -32001 and abort the request's execution",
			ConfigKey:    &cfg.MaxGetTransactionExecutionDuration,
			DefaultValue: 5 * time.Second,
		},
		{
			TomlKey:      strutils.KebabToConstantCase("max-send-transaction-execution-duration"),
			Usage:        "The maximum duration of time allowed for processing a sendTransaction request. When that time elapses, the rpc server would return -32001 and abort the request's execution",
			ConfigKey:    &cfg.MaxSendTransactionExecutionDuration,
			DefaultValue: 15 * time.Second,
		},
		{
			TomlKey:      strutils.KebabToConstantCase("max-simulate-transaction-execution-duration"),
			Usage:        "The maximum duration of time allowed for processing a simulateTransaction request. When that time elapses, the rpc server would return -32001 and abort the request's execution",
			ConfigKey:    &cfg.MaxSimulateTransactionExecutionDuration,
			DefaultValue: 15 * time.Second,
		},
	}
	return *cfg.optionsCache
}

type errMissingRequiredOption struct {
	strErr string
	usage  string
}

func (e errMissingRequiredOption) Error() string {
	return e.strErr
}

func required(option *ConfigOption) error {
	switch reflect.ValueOf(option.ConfigKey).Elem().Kind() {
	case reflect.Slice:
		if reflect.ValueOf(option.ConfigKey).Elem().Len() > 0 {
			return nil
		}
	default:
		if !reflect.ValueOf(option.ConfigKey).Elem().IsZero() {
			return nil
		}
	}

	waysToSet := []string{}
	if option.Name != "" && option.Name != "-" {
		waysToSet = append(waysToSet, fmt.Sprintf("specify --%s on the command line", option.Name))
	}
	if option.EnvVar != "" && option.EnvVar != "-" {
		waysToSet = append(waysToSet, fmt.Sprintf("set the %s environment variable", option.EnvVar))
	}

	if tomlKey, hasTomlKey := option.getTomlKey(); hasTomlKey {
		waysToSet = append(waysToSet, fmt.Sprintf("set %s in the config file", tomlKey))
	}

	advice := ""
	switch len(waysToSet) {
	case 1:
		advice = fmt.Sprintf(" Please %s.", waysToSet[0])
	case 2:
		advice = fmt.Sprintf(" Please %s or %s.", waysToSet[0], waysToSet[1])
	case 3:
		advice = fmt.Sprintf(" Please %s, %s, or %s.", waysToSet[0], waysToSet[1], waysToSet[2])
	}

	return errMissingRequiredOption{strErr: fmt.Sprintf("%s is required.%s", option.Name, advice), usage: option.Usage}
}

func positive(option *ConfigOption) error {
	switch v := option.ConfigKey.(type) {
	case *int, *int8, *int16, *int32, *int64:
		if reflect.ValueOf(v).Elem().Int() <= 0 {
			return fmt.Errorf("%s must be positive", option.Name)
		}
	case *uint, *uint8, *uint16, *uint32, *uint64:
		if reflect.ValueOf(v).Elem().Uint() <= 0 {
			return fmt.Errorf("%s must be positive", option.Name)
		}
	default:
		return fmt.Errorf("%s is not a positive integer", option.Name)
	}
	return nil
}
