FROM teamfruit/nginx-fcgiwrap

RUN apt-get update && apt-get upgrade -y
RUN apt-get install git-core fcgiwrap -y
COPY nginx.conf /etc/nginx/conf.d/default.conf
