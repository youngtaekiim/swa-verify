#!/bin/bash

# Create certificates directory
mkdir -p certs

# Generate CA private key
openssl genrsa -out certs/ca.key 4096

# Generate CA certificate
openssl req -new -x509 -key certs/ca.key -sha256 -subj "/C=US/ST=CA/O=TestOrg/CN=TestCA" -days 3650 -out certs/ca.crt

# Generate server private key
openssl genrsa -out certs/server.key 4096

# Create certificate config with SAN
cat > certs/server.conf <<EOF
[req]
distinguished_name = req_distinguished_name
req_extensions = v3_req
prompt = no

[req_distinguished_name]
C = US
ST = CA
O = TestOrg
CN = localhost

[v3_req]
keyUsage = keyEncipherment, dataEncipherment
extendedKeyUsage = serverAuth
subjectAltName = @alt_names

[alt_names]
DNS.1 = localhost
DNS.2 = *.localhost
IP.1 = 127.0.0.1
IP.2 = ::1
EOF

# Create certificate signing request for server
openssl req -new -key certs/server.key -config certs/server.conf -out certs/server.csr

# Generate server certificate signed by CA
openssl x509 -req -in certs/server.csr -CA certs/ca.crt -CAkey certs/ca.key -CAcreateserial -out certs/server.crt -days 365 -sha256 -extensions v3_req -extfile certs/server.conf

# Clean up temporary files
rm certs/server.csr certs/server.conf

echo "Certificates generated successfully with SAN!"
echo "Files created:"
echo "  certs/ca.crt - Certificate Authority"
echo "  certs/ca.key - CA Private Key"
echo "  certs/server.crt - Server Certificate"
echo "  certs/server.key - Server Private Key"