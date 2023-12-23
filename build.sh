cargo build --release
cp ./target/release/numberpad_rs ./build/numberpad_rs_binaries/
cp ./numberpad_rs.service ./build/numberpad_rs_binaries/
tar -zcvf numberpad_rs_binaries.tar.gz ./build/numberpad_rs_binaries/ 