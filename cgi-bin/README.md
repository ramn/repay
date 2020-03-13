CGI runner
==========

A Docker image with Apache2 that runs cgi-bin applications.


Build the image
---------------

```bash
docker build -f Dockerfile -t cgirunner:latest .
```

Run the server
--------------
With `--restart=always` Docker will make sure to restart this container after
the host OS is rebooted.

```bash
docker run --restart=always -d \
  -p 127.0.0.1:8080:80 \
  --hostname example.com \
  --name cgirunner \
  -v "$PWD":/usr/lib/cgi-bin \
  cgirunner:latest
```

Set up Apache to reverse proxy to the docker container
------------------------------------------------------

```
# Save in a file:
# /etc/apache2/sites-enabled/my-cgi-proxy.conf

<VirtualHost *:80>
        ServerName example.com
        ProxyPass "/" "http://localhost:8080/"

        ErrorLog ${APACHE_LOG_DIR}/error-example.com.log
        LogLevel warn
        CustomLog ${APACHE_LOG_DIR}/access-example.com.log combined
        ServerSignature On
</VirtualHost>
```
