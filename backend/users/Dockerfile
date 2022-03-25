FROM --platform=linux/amd64 c5-project-api-base:latest

ARG APP_USER=appuser
ARG APP_HOME=/home/${APP_USER}

EXPOSE 8080

COPY --from=c5-project-rust-build:latest --chown=${APP_USER}:${APP_USER} \
     /usr/src/builddir/target/release/users ${APP_HOME}/users

COPY Rocket.toml ${APP_HOME}/

USER $APP_USER
WORKDIR ${APP_HOME}

CMD ["./users"]
