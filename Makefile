run:
	@cargo run

build:
	@cargo build

build-image:
	@docker build -t mechardo3d .

run-image: build-image
	@docker run -p 3000:3000 mechardo3d -d

stop-image:
	@docker ps -a -q --filter ancestor=mechardo3d > temp.txt && \
	if exist temp.txt ( \
		for /f %%i in (temp.txt) do docker rm -f %%i \
	) && if exist temp.txt (del temp.txt)
