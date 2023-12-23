mkdir build
cd build
tar -xvf ../numberpad_rs_binaries.tar.gz
cp numberpad_rs_binaries/* .

sudo cp numberpad_rs /usr/bin
sudo cp numberpad_rs.service /etc/systemd/system/