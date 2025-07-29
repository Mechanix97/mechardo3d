run:
	@cargo run

build:
	@cargo build

build-image:
	@docker build -t mechardo3d .

run-image: build-image 
	@docker run -p 3000:3000 --name mechardo3d -d mechardo3d

stop-image:
	@docker stop mechardo3d
	@docker remove mechardo3d

run-prod:
	@docker compose up -d
