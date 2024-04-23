rm image.tar
rm rust-ms.zip
docker build . -t rust-ms
docker save rust-ms -o image.tar
zip rust-ms image.tar cumulocity.json
