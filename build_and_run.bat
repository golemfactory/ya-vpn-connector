docker-compose build novpn
docker-compose kill novpn
docker-compose up -d novpn
docker-compose logs -f novpn