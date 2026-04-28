ocker_compose('docker-compose.yaml')

watch_settings(ignore=[
    '**/target',
    '**/node_modules',
    '**/.venv',
    '**/__pycache__',
    'client/.output',
    'client/.tanstack',
])

local_resource(
    'server',
    cmd='exit 0',
    serve_cmd='cargo run --features dotenv --manifest-path server/Cargo.toml -- run-server',
    deps=['server/src', 'server/Cargo.toml', 'migration/src'],
    labels=['service']
)

local_resource(
    'client',
    cmd='exit 0',
    serve_cmd='pnpm --prefix client dev',
    resource_deps=['server'],
    labels=['service']
    # deps=['client/src', 'client/package.json', 'client/vite.config.ts'],
)

local_resource(
    'ml-worker',
    cmd='exit 0',
    serve_cmd='.venv/bin/python3.13 ml-worker/main.py',
    resource_deps=['server'],
    deps=['ml-worker'],
    labels=['service']
)

local_resource(
    'run-migrations',
    cmd='cargo run --manifest-path migration/Cargo.toml',
    serve_cmd='exit 0', # Run once and stop
    auto_init=False,    # Don't run on start
    trigger_mode=TRIGGER_MODE_MANUAL,
    labels=['utils']
)
