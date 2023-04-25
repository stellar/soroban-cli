package config

import (
	"fmt"
	"go/types"
	"os"
	"os/exec"
	"reflect"
	"runtime"
	"strings"
	"time"

	"github.com/sirupsen/logrus"

	"github.com/stellar/go/network"
	"github.com/stellar/go/support/errors"
	"github.com/stellar/soroban-tools/cmd/soroban-rpc/internal/ledgerbucketwindow"
)

// ConfigOption is a complete description of the configuration of a command line option
type ConfigOption struct {
	Name           string                                 // e.g. "database-url"
	EnvVar         string                                 // e.g. "DATABASE_URL". Defaults to uppercase/underscore representation of name
	TomlKey        string                                 // e.g. "DATABASE_URL". Defaults to uppercase/underscore representation of name. - to omit from toml
	Usage          string                                 // Help text
	OptType        types.BasicKind                        // The type of this option, e.g. types.Bool
	DefaultValue   interface{}                            // A default if no option is provided. Omit or set to `nil` if no default
	ConfigKey      interface{}                            // Pointer to the final key in the linked Config struct
	CustomSetValue func(*ConfigOption, interface{}) error // Optional function for custom validation/transformation
	Validate       func(*ConfigOption) error              // Function called after loading all options, to validate the configuration
}

// ConfigOptions is a group of ConfigOptions that can be for convenience
// initialized and set at the same time.
type ConfigOptions []*ConfigOption

func (cfg *Config) options() ConfigOptions {
	defaultStellarCoreBinaryPath, _ := exec.LookPath("stellar-core")
	return ConfigOptions{
		{
			Name:      "config-path",
			EnvVar:    "SOROBAN_RPC_CONFIG_PATH",
			TomlKey:   "-",
			Usage:     "File path to the toml configuration file",
			OptType:   types.String,
			ConfigKey: &cfg.ConfigPath,
		},
		{
			Name:         "config-strict",
			EnvVar:       "SOROBAN_RPC_CONFIG_STRICT",
			TomlKey:      "STRICT",
			Usage:        "Enable strict toml configuration file parsing",
			OptType:      types.Bool,
			ConfigKey:    &cfg.Strict,
			DefaultValue: false,
		},
		{
			Name:         "endpoint",
			Usage:        "Endpoint to listen and serve on",
			OptType:      types.String,
			ConfigKey:    &cfg.Endpoint,
			DefaultValue: "localhost:8000",
		},
		{
			Name:      "admin-endpoint",
			Usage:     "Admin endpoint to listen and serve on. WARNING: this should not be accessible from the Internet and does not use TLS. \"\" (default) disables the admin server",
			OptType:   types.String,
			ConfigKey: &cfg.AdminEndpoint,
		},
		{
			Name:      "stellar-core-url",
			Usage:     "URL used to query Stellar Core (local captive core by default)",
			OptType:   types.String,
			ConfigKey: &cfg.StellarCoreURL,
		},
		{
			Name:           "stellar-core-timeout-seconds",
			Usage:          "Timeout used when submitting requests to stellar-core",
			OptType:        types.String,
			ConfigKey:      &cfg.CoreRequestTimeout,
			DefaultValue:   Duration{2 * time.Second},
			CustomSetValue: parseValue(cfg.CoreRequestTimeout.UnmarshalTOML),
		},
		{
			Name:         "stellar-captive-core-http-port",
			Usage:        "HTTP port for Captive Core to listen on (0 disables the HTTP server)",
			OptType:      types.Uint,
			ConfigKey:    &cfg.CaptiveCoreHTTPPort,
			DefaultValue: uint(11626),
		},
		{
			Name:         "log-level",
			Usage:        "minimum log severity (debug, info, warn, error) to log",
			OptType:      types.String,
			ConfigKey:    &cfg.LogLevel,
			DefaultValue: logrus.InfoLevel,
			CustomSetValue: func(option *ConfigOption, i interface{}) error {
				switch v := i.(type) {
				case string:
					ll, err := logrus.ParseLevel(v)
					if err != nil {
						return fmt.Errorf("could not parse %s: %v", option.Name, v)
					}
					cfg.LogLevel = ll
					return nil
				default:
					return fmt.Errorf("could not parse %s: %v", option.Name, v)
				}
			},
		},
		{
			Name:           "log-format",
			Usage:          "format used for output logs (json or text)",
			OptType:        types.String,
			ConfigKey:      &cfg.LogFormat,
			DefaultValue:   LogFormatText,
			CustomSetValue: parseValue(cfg.LogFormat.UnmarshalTOML),
		},
		{
			Name:         "stellar-core-binary-path",
			Usage:        "path to stellar core binary",
			OptType:      types.String,
			ConfigKey:    &cfg.StellarCoreBinaryPath,
			DefaultValue: defaultStellarCoreBinaryPath,
			Validate:     required,
		},
		{
			Name:      "captive-core-config-path",
			Usage:     "path to additional configuration for the Stellar Core configuration file used by captive core. It must, at least, include enough details to define a quorum set",
			OptType:   types.String,
			ConfigKey: &cfg.CaptiveCoreConfigPath,
			Validate:  required,
		},
		{
			Name:      "captive-core-storage-path",
			Usage:     "Storage location for Captive Core bucket data",
			OptType:   types.String,
			ConfigKey: &cfg.CaptiveCoreStoragePath,
			CustomSetValue: func(option *ConfigOption, i interface{}) error {
				switch v := i.(type) {
				case string:
					if v == "" || v == "." {
						cwd, err := os.Getwd()
						if err != nil {
							return fmt.Errorf("Unable to determine the current directory: %s", err)
						}
						v = cwd
					}
					cfg.CaptiveCoreStoragePath = v
					return nil
				case nil:
					cwd, err := os.Getwd()
					if err != nil {
						return fmt.Errorf("Unable to determine the current directory: %s", err)
					}
					cfg.CaptiveCoreStoragePath = cwd
					return nil
				default:
					return fmt.Errorf("Could not parse %s: %v", option.Name, v)
				}
			},
		},
		{
			Name:         "captive-core-use-db",
			Usage:        "informs captive core to use on disk mode. the db will by default be created in current runtime directory of soroban-rpc, unless DATABASE=<path> setting is present in captive core config file.",
			OptType:      types.Bool,
			ConfigKey:    &cfg.CaptiveCoreUseDB,
			DefaultValue: false,
		},
		{
			Name:      "history-archive-urls",
			Usage:     "comma-separated list of stellar history archives to connect with",
			OptType:   types.String,
			ConfigKey: &cfg.HistoryArchiveURLs,
			CustomSetValue: func(option *ConfigOption, i interface{}) error {
				switch v := i.(type) {
				case string:
					if v == "" {
						cfg.HistoryArchiveURLs = nil
					} else {
						cfg.HistoryArchiveURLs = strings.Split(v, ",")
					}
					return nil
				case []string:
					cfg.HistoryArchiveURLs = v
					return nil
				default:
					return fmt.Errorf("Could not parse %s: %v", option.Name, v)
				}
			},
			Validate: required,
		},
		{
			Name:      "friendbot-url",
			Usage:     "The friendbot URL to be returned by getNetwork endpoint",
			OptType:   types.String,
			ConfigKey: &cfg.FriendbotURL,
		},
		{
			Name:         "network-passphrase",
			Usage:        "Network passphrase of the Stellar network transactions should be signed for",
			OptType:      types.String,
			ConfigKey:    &cfg.NetworkPassphrase,
			DefaultValue: network.FutureNetworkPassphrase,
			Validate:     required,
		},
		{
			Name:         "db-path",
			Usage:        "SQLite DB path",
			OptType:      types.String,
			ConfigKey:    &cfg.SQLiteDBPath,
			DefaultValue: "soroban_rpc.sqlite",
		},
		{
			Name:           "ingestion-timeout",
			Usage:          "Ingestion Timeout when bootstrapping data (checkpoint and in-memory initialization) and preparing ledger reads",
			OptType:        types.String,
			ConfigKey:      &cfg.IngestionTimeout,
			DefaultValue:   Duration{30 * time.Minute},
			CustomSetValue: parseValue(cfg.IngestionTimeout.UnmarshalTOML),
		},
		{
			Name:         "checkpoint-frequency",
			Usage:        "establishes how many ledgers exist between checkpoints, do NOT change this unless you really know what you are doing",
			OptType:      types.Uint32,
			ConfigKey:    &cfg.CheckpointFrequency,
			DefaultValue: uint32(64),
		},
		{
			Name: "event-retention-window",
			Usage: fmt.Sprintf("configures the event retention window expressed in number of ledgers,"+
				" the default value is %d which corresponds to about 24 hours of history", ledgerbucketwindow.DefaultEventLedgerRetentionWindow),
			OptType:        types.Uint32,
			ConfigKey:      &cfg.EventLedgerRetentionWindow,
			DefaultValue:   uint32(ledgerbucketwindow.DefaultEventLedgerRetentionWindow),
			CustomSetValue: parseValue(cfg.EventLedgerRetentionWindow.UnmarshalTOML),
		},
		{
			Name: "transaction-retention-window",
			Usage: "configures the transaction retention window expressed in number of ledgers," +
				" the default value is 1440 which corresponds to about 2 hours of history",
			OptType:        types.Uint32,
			ConfigKey:      &cfg.TransactionLedgerRetentionWindow,
			DefaultValue:   uint32(1440),
			CustomSetValue: parseValue(cfg.TransactionLedgerRetentionWindow.UnmarshalTOML),
		},
		{
			Name:         "max-events-limit",
			Usage:        "Maximum amount of events allowed in a single getEvents response",
			OptType:      types.Uint,
			ConfigKey:    &cfg.MaxEventsLimit,
			DefaultValue: uint(10000),
		},
		{
			Name:         "default-events-limit",
			Usage:        "Default cap on the amount of events included in a single getEvents response",
			OptType:      types.Uint,
			ConfigKey:    &cfg.DefaultEventsLimit,
			DefaultValue: uint(100),
			Validate: func(co *ConfigOption) error {
				if cfg.DefaultEventsLimit > cfg.MaxEventsLimit {
					return fmt.Errorf(
						"default-events-limit (%v) cannot exceed max-events-limit (%v)\n",
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
			OptType:        types.String,
			ConfigKey:      &cfg.MaxHealthyLedgerLatency,
			DefaultValue:   Duration{30 * time.Second},
			CustomSetValue: parseValue(cfg.MaxHealthyLedgerLatency.UnmarshalTOML),
		},
		{
			Name:           "preflight-worker-count",
			Usage:          "Number of workers (read goroutines) used to compute preflights for the simulateTransaction endpoint. Defaults to the number of CPUs.",
			OptType:        types.Uint,
			ConfigKey:      &cfg.PreflightWorkerCount,
			DefaultValue:   uint(runtime.NumCPU()),
			CustomSetValue: parseValue(cfg.PreflightWorkerCount.UnmarshalTOML),
		},
		{
			Name:         "preflight-worker-queue-size",
			Usage:        "Maximum number of outstanding preflight requests for the simulateTransaction endpoint. Defaults to the number of CPUs.",
			OptType:      types.Uint,
			ConfigKey:    &cfg.PreflightWorkerQueueSize,
			DefaultValue: uint(runtime.NumCPU()),
		},
	}
}

func (options ConfigOptions) Validate() error {
	for _, option := range options {
		if option.Validate != nil {
			err := option.Validate(option)
			if err != nil {
				return errors.Wrap(err, fmt.Sprintf("Invalid config value for %s", option.Name))
			}
		}
	}

	return nil
}

func required(option *ConfigOption) error {
	if !reflect.ValueOf(option.ConfigKey).Elem().IsZero() {
		return nil
	}

	waysToSet := []string{}
	if option.Name != "" && option.Name != "-" {
		waysToSet = append(waysToSet, fmt.Sprintf("specify --%s on the command line", option.Name))
	}
	if option.EnvVar != "" && option.EnvVar != "-" {
		waysToSet = append(waysToSet, fmt.Sprintf("set the %s environment variable", option.EnvVar))
	}
	if option.TomlKey != "" && option.TomlKey != "-" {
		waysToSet = append(waysToSet, fmt.Sprintf("set %s in the config file", option.EnvVar))
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

	return fmt.Errorf("Invalid config: %s is required.%s", option.Name, advice)
}

func parseValue(f func(interface{}) error) func(*ConfigOption, interface{}) error {
	return func(option *ConfigOption, i interface{}) error {
		return errors.Wrapf(f(i), "Could not parse %s", option.Name)
	}
}
