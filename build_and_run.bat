docker-compose -f docker-compose-dev.yml build novpn
docker-compose -f docker-compose-dev.yml kill novpn
docker-compose -f docker-compose-dev.yml up -d novpn
docker-compose -f docker-compose-dev.yml logs -f novpn