
run:
	@cargo run

build:
	@cargo build

build-image:
	@docker build -t mechardo3d .

run-image:
	@docker run -p 3000:3000 -d mechardo3d

stop-image:
	@docker ps -q --filter ancestor=mechardo3d > temp.txt && \
	if exist temp.txt ( \
		for /f %%i in (temp.txt) do (docker stop %%i && docker rm %%i) \
	) && del temp.txt
