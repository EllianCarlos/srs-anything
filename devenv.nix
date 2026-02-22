{ pkgs, ... }:
{
  dotenv.enable = true;

  packages = [
    pkgs.git
    pkgs.gitleaks
    pkgs.openssl
    pkgs.pkg-config
    pkgs.cargo-llvm-cov
    pkgs.cargo-nextest
    pkgs.sqlx-cli
    pkgs.llvmPackages_19.llvm
  ];

  languages.rust = {
    enable = true;
  };

  languages.javascript = {
    enable = true;
    npm.enable = true;
  };

  services.postgres = {
    enable = true;
    port = 5433;
    listen_addresses = "127.0.0.1";
    initialDatabases = [
      {
        name = "srs_anything";
        user = "srs";
        pass = "srs";
        initialSQL = ''
          GRANT USAGE, CREATE ON SCHEMA public TO srs;
        '';
      }
    ];
  };

  env = {
    DATABASE_URL = "postgres://srs:srs@127.0.0.1:5433/srs_anything";
    APP_ENV = "development";
    SRS_CONFIG_PATH = "config/srs_schedule.yaml";
    SRS_PROFILE = "test";
    JWT_ISSUER = "srs-anything";
    JWT_AUDIENCE = "srs-anything-web";
    JWT_EXPIRATION_SECS = "2592000";
    COOKIE_SECURE = "false";
    ALLOWED_ORIGINS = "http://localhost:5173";
    LLVM_COV = "${pkgs.llvmPackages_19.llvm}/bin/llvm-cov";
    LLVM_PROFDATA = "${pkgs.llvmPackages_19.llvm}/bin/llvm-profdata";
  };

  scripts.lint.exec = ''
    set -euo pipefail
    if [ -d backend ]; then
      (cd backend && cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings)
    fi
    if [ -d frontend ]; then
      (cd frontend && npm run lint && npm run typecheck)
    fi
  '';

  scripts.test.exec = ''
    set -euo pipefail
    if [ -d backend ]; then
      (cd backend && cargo test --all-features)
    fi
    if [ -d frontend ]; then
      (cd frontend && npm run test -- --run)
    fi
  '';

  scripts.coverage.exec = ''
    set -euo pipefail
    if [ -d backend ]; then
      (cd backend && mkdir -p coverage)
      (cd backend && cargo llvm-cov --workspace --lcov --output-path coverage/lcov.info)
    fi
    if [ -d frontend ]; then
      (cd frontend && mkdir -p coverage)
      (cd frontend && npm run coverage)
    fi
  '';

  scripts.secrets.exec = ''
    set -euo pipefail
    gitleaks git --verbose .
  '';

  scripts.ci-local.exec = ''
    set -euo pipefail
    lint
    secrets
    test
    coverage
  '';
}
