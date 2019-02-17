FROM debian:latest
RUN apt-get update && apt-get install -y apache2 \
  && rm -rf /var/lib/apt/lists/* \
  && a2enmod cgi && service apache2 restart
COPY serve-cgi-bin.conf /etc/apache2/conf-available/serve-cgi-bin.conf
EXPOSE 80
CMD apachectl -D FOREGROUND
