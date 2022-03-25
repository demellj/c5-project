FROM --platform=linux/amd64 debian:stable-slim

ARG APP_USER=appuser
ARG APP_HOME=/home/${APP_USER}

ENV TZ=Etc/UTC \
    APP_USER=${APP_USER}

RUN apt-get update \
    && apt-get install --no-install-recommends -y ca-certificates tzdata libpq5 \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP_HOME} \
	 && touch "${APP_HOME}/.env" 

