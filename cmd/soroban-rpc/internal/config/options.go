package config

import (
	"fmt"
	"go/types"
	"os"
	"os/exec"
	"reflect"
	"runtime"
	"time"

	"github.com/sirupsen/logrus"

	"github.com/stellar/go/network"
	"github.com/stellar/go/support/errors"
	"github.com/stellar/soroban-tools/cmd/soroban-rpc/internal/ledgerbucketwindow"
)

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
			OptType:   types.String,
			ConfigKey: &cfg.ConfigPath,
		},
		{
			Name:         "config-strict",
			EnvVar:       "SOROBAN_RPC_CONFIG_STRICT",
			TomlKey:      "STRICT",
			Usage:        "Enable strict toml configuration file parsing. This will prevent unknown fields in the config toml from being parsed.",
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
			Name:           "stellar-core-timeout",
			Usage:          "Timeout used when submitting requests to stellar-core",
			OptType:        types.String,
			ConfigKey:      &cfg.CoreRequestTimeout,
			DefaultValue:   2 * time.Second,
			CustomSetValue: parseDuration,
			MarshalTOML:    marshalDuration,
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
			OptType:      types.String,
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
				return cfg.LogFormat.String(), nil
			},
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
			Name:         "captive-core-use-db",
			Usage:        "informs captive core to use on disk mode. the db will by default be created in current runtime directory of soroban-rpc, unless DATABASE=<path> setting is present in captive core config file.",
			OptType:      types.Bool,
			ConfigKey:    &cfg.CaptiveCoreUseDB,
			DefaultValue: false,
		},
		{
			Name:           "history-archive-urls",
			Usage:          "comma-separated list of stellar history archives to connect with",
			OptType:        types.String,
			ConfigKey:      &cfg.HistoryArchiveURLs,
			CustomSetValue: parseStringArray,
			Validate:       required,
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
			DefaultValue:   30 * time.Minute,
			CustomSetValue: parseDuration,
			MarshalTOML:    marshalDuration,
		},
		{
			Name:           "checkpoint-frequency",
			Usage:          "establishes how many ledgers exist between checkpoints, do NOT change this unless you really know what you are doing",
			OptType:        types.Uint32,
			ConfigKey:      &cfg.CheckpointFrequency,
			DefaultValue:   uint32(64),
			CustomSetValue: parseUint32,
		},
		{
			Name: "event-retention-window",
			Usage: fmt.Sprintf("configures the event retention window expressed in number of ledgers,"+
				" the default value is %d which corresponds to about 24 hours of history", ledgerbucketwindow.DefaultEventLedgerRetentionWindow),
			OptType:        types.Uint32,
			ConfigKey:      &cfg.EventLedgerRetentionWindow,
			DefaultValue:   uint32(ledgerbucketwindow.DefaultEventLedgerRetentionWindow),
			Validate:       positive,
			CustomSetValue: parseUint32,
		},
		{
			Name: "transaction-retention-window",
			Usage: "configures the transaction retention window expressed in number of ledgers," +
				" the default value is 1440 which corresponds to about 2 hours of history",
			OptType:        types.Uint32,
			ConfigKey:      &cfg.TransactionLedgerRetentionWindow,
			DefaultValue:   uint32(1440),
			Validate:       positive,
			CustomSetValue: parseUint32,
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
			OptType:        types.String,
			ConfigKey:      &cfg.MaxHealthyLedgerLatency,
			DefaultValue:   30 * time.Second,
			CustomSetValue: parseDuration,
			MarshalTOML:    marshalDuration,
		},
		{
			Name:         "preflight-worker-count",
			Usage:        "Number of workers (read goroutines) used to compute preflights for the simulateTransaction endpoint. Defaults to the number of CPUs.",
			OptType:      types.Uint,
			ConfigKey:    &cfg.PreflightWorkerCount,
			DefaultValue: uint(runtime.NumCPU()),
			Validate:     positive,
		},
		{
			Name:         "preflight-worker-queue-size",
			Usage:        "Maximum number of outstanding preflight requests for the simulateTransaction endpoint. Defaults to the number of CPUs.",
			OptType:      types.Uint,
			ConfigKey:    &cfg.PreflightWorkerQueueSize,
			DefaultValue: uint(runtime.NumCPU()),
			Validate:     positive,
		},
	}
	return *cfg.optionsCache
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

	return fmt.Errorf("%s is required.%s", option.Name, advice)
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

func marshalDuration(option *ConfigOption) (interface{}, error) {
	if option.ConfigKey == nil {
		return nil, nil
	}
	return option.ConfigKey.(*time.Duration).String(), nil
}
