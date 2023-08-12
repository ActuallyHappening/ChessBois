set -exu

# Found the following certs:
#   Certificate Name: actually-happening.com
#     Serial Number: 37c9cd60d56013c9040af230cc50edb9ac3
#     Key Type: ECDSA
#     Domains: actually-happening.com
#     Expiry Date: 2023-11-09 06:02:53+00:00 (VALID: 89 days)
#     Certificate Path: /etc/letsencrypt/live/actually-happening.com/fullchain.pem
#     Private Key Path: /etc/letsencrypt/live/actually-happening.com/privkey.pem

simple-http-server --port 80 --index --compress *