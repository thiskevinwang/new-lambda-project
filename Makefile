build:
	cargo lambda build --release

deploy:
	cargo lambda deploy --enable-function-url

dev:
	cargo lambda watch

docs:
	open https://www.cargo-lambda.info/

db:
	docker run --name some-postgres -p 5432:5432 -e POSTGRES_PASSWORD=mysecretpassword postgres