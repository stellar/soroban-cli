package config

import (
	"os"
	"time"

	"github.com/sirupsen/logrus"
	"github.com/spf13/pflag"
)

// Config represents the configuration of a soroban-rpc server
type Config struct {
	ConfigPath string

	Strict bool

	StellarCoreURL         string
	CaptiveCoreStoragePath string
	StellarCoreBinaryPath  string
	CaptiveCoreConfigPath  string
	CaptiveCoreHTTPPort    uint

	Endpoint                                    string
	AdminEndpoint                               string
	CheckpointFrequency                         uint32
	CoreRequestTimeout                          time.Duration
	DefaultEventsLimit                          uint
	EventLedgerRetentionWindow                  uint32
	FriendbotURL                                string
	HistoryArchiveURLs                          []string
	IngestionTimeout                            time.Duration
	LogFormat                                   LogFormat
	LogLevel                                    logrus.Level
	MaxEventsLimit                              uint
	MaxHealthyLedgerLatency                     time.Duration
	NetworkPassphrase                           string
	PreflightWorkerCount                        uint
	PreflightWorkerQueueSize                    uint
	PreflightEnableDebug                        bool
	SQLiteDBPath                                string
	TransactionLedgerRetentionWindow            uint32
	LedgerRetentionWindow                       uint32
	RequestBacklogGlobalQueueLimit              uint
	RequestBacklogGetHealthQueueLimit           uint
	RequestBacklogGetEventsQueueLimit           uint
	RequestBacklogGetNetworkQueueLimit          uint
	RequestBacklogGetLatestLedgerQueueLimit     uint
	RequestBacklogGetLedgerEntriesQueueLimit    uint
	RequestBacklogGetTransactionQueueLimit      uint
	RequestBacklogSendTransactionQueueLimit     uint
	RequestBacklogSimulateTransactionQueueLimit uint
	RequestExecutionWarningThreshold            time.Duration
	MaxRequestExecutionDuration                 time.Duration
	MaxGetHealthExecutionDuration               time.Duration
	MaxGetEventsExecutionDuration               time.Duration
	MaxGetNetworkExecutionDuration              time.Duration
	MaxGetLatestLedgerExecutionDuration         time.Duration
	MaxGetLedgerEntriesExecutionDuration        time.Duration
	MaxGetTransactionExecutionDuration          time.Duration
	MaxSendTransactionExecutionDuration         time.Duration
	MaxSimulateTransactionExecutionDuration     time.Duration

	// We memoize these, so they bind to pflags correctly
	optionsCache *ConfigOptions
	flagset      *pflag.FlagSet
}

func (cfg *Config) SetValues(lookupEnv func(string) (string, bool)) error {
	// We start with the defaults
	if err := cfg.loadDefaults(); err != nil {
		return err
	}

	// Then we load from the environment variables and cli flags, to try to find
	// the config file path
	if err := cfg.loadEnv(lookupEnv); err != nil {
		return err
	}
	if err := cfg.loadFlags(); err != nil {
		return err
	}

	// If we specified a config file, we load that
	if cfg.ConfigPath != "" {
		// Merge in the config file flags
		if err := cfg.loadConfigPath(); err != nil {
			return err
		}

		// Load from cli flags and environment variables again, to overwrite what we
		// got from the config file
		if err := cfg.loadEnv(lookupEnv); err != nil {
			return err
		}
		if err := cfg.loadFlags(); err != nil {
			return err
		}
	}

	return nil
}

// loadDefaults populates the config with default values
func (cfg *Config) loadDefaults() error {
	for _, option := range cfg.options() {
		if option.DefaultValue != nil {
			if err := option.setValue(option.DefaultValue); err != nil {
				return err
			}
		}
	}
	return nil
}

// loadEnv populates the config with values from the environment variables
func (cfg *Config) loadEnv(lookupEnv func(string) (string, bool)) error {
	for _, option := range cfg.options() {
		key, ok := option.getEnvKey()
		if !ok {
			continue
		}
		value, ok := lookupEnv(key)
		if !ok {
			continue
		}
		if err := option.setValue(value); err != nil {
			return err
		}
	}
	return nil
}

// loadFlags populates the config with values from the cli flags
func (cfg *Config) loadFlags() error {
	for _, option := range cfg.options() {
		if option.flag == nil || !option.flag.Changed {
			continue
		}
		val, err := option.GetFlag(cfg.flagset)
		if err != nil {
			return err
		}
		if err := option.setValue(val); err != nil {
			return err
		}
	}
	return nil
}

// loadConfigPath loads a new config from a toml file at the given path. Strict
// mode will return an error if there are any unknown toml variables set. Note,
// strict-mode can also be set by putting `STRICT=true` in the config.toml file
// itself.
func (cfg *Config) loadConfigPath() error {
	file, err := os.Open(cfg.ConfigPath)
	if err != nil {
		return err
	}
	defer file.Close()
	return parseToml(file, cfg.Strict, cfg)
}

func (cfg *Config) Validate() error {
	return cfg.options().Validate()
}
