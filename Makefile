# Минимум команд для запуска проекта

.PHONY: install validator validator-metaplex build deploy deploy-devnet deploy-oracle-devnet deploy-minter-devnet init init-devnet deploy-oracle deploy-minter backend backend-devnet frontend kill-frontend test prepare check-rs lint-rs lint

install:
	cd programs-tests && yarn install
	cd modules/frontend && yarn install

# Обычный локальный валидатор (без Metaplex)
validator:
	solana-test-validator --reset

# Валидатор с клоном Metaplex Token Metadata (для отображения имени/тикера/картинки в кошельке)
validator-metaplex:
	solana-test-validator --clone-upgradeable-program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s --url https://api.mainnet-beta.solana.com

build:
	anchor build

# Деплоит оба контракта на localnet (сначала запусти make validator в другом терминале)
deploy: build
	anchor deploy --provider.cluster localnet

# Devnet: переключись (solana config set-url devnet), пополни (solana airdrop 2), затем деплой обоих контрактов:
deploy-devnet: deploy-oracle-devnet deploy-minter-devnet

# Devnet: деплой только оракула или только минтера
deploy-oracle-devnet: build
	anchor deploy --program-name sol_usd_oracle --provider.cluster devnet

deploy-minter-devnet: build
	anchor deploy --program-name token_minter --provider.cluster devnet

# Инициализация оракула и минтера (localnet по умолчанию)
init:
	cargo run -p programs-tests --example init-accounts

get-oracle-pda:
	cargo run -p programs-tests --example get-oracle-pda

# Инициализация на devnet (после make deploy-devnet)
init-devnet:
	RPC_URL=https://api.devnet.solana.com node scripts/init-local.js

# Localnet: деплой только оракула или только минтера (сначала make validator)
deploy-oracle: build
	anchor deploy --program-name sol_usd_oracle --provider.cluster localnet

deploy-minter: build
	anchor deploy --program-name token_minter --provider.cluster localnet

# Backend для localnet (читает backend/.env; нужны SOLANA_RPC_HTTP, SOLANA_RPC_WS)
backend:
	cd modules/backend && cargo run

# Backend для devnet (подставляет RPC devnet; ORACLE_STATE_PUBKEY и т.д. из .env)
backend-devnet:
	cd modules/backend && SOLANA_RPC_HTTP=https://api.devnet.solana.com SOLANA_RPC_WS=wss://api.devnet.solana.com cargo run

# Освободить порт 7001 (если занят старым процессом фронта)
kill-frontend:
	-lsof -ti:7001 | xargs kill 2>/dev/null || true

frontend: kill-frontend
	cd modules/frontend && yarn dev

test:
	cd programs-tests && yarn run ts-mocha -p ./tsconfig.json -t 1000000 "tests/**/*.ts"
	cargo test
	#anchor test WIP


prepare:
	./configs/git/setup.sh