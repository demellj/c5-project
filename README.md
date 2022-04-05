# Image feed application

This is the backend for an image feed micro-application, where users can share images with one another.

There are a few microservices for this proof of concept mirco-application, namely:

* feed
  * Provides ability to get feeds, post new feeds, modify existing feeds and delete feeds.
  * Additionally provides ability to get a feeds thumbnail or all thumbnails generated.
  * Supports pagination.
  * Only users who own a feed can modify or delete those feeds.
* users
  * Authenticates and authorizes users to the feed application
  * Provides ability to register new users with application
* imgproc
  * Generates thumbnails for image feeds
  * Removes thumbnail when feeds is deleted
* reverseproxy
  * Provides a reverse-proxy services fronting the users and feed API services

## The stack

The backend is entirely written in `Rust`.  The following frameworks and libraries are utilized to build this project:

* The API backends are written using the [Actix Web](https://actix.rs/).
* Data ORM is provided by [Diesel](http://diesel.rs/).
* AWS specific functionality by the offical [AWS SDK for Rust](https://awslabs.github.io/aws-sdk-rust/).
* Asynchronous runtime using [Tokio](https://tokio.rs/).

Notable libraries:

* [jsonwebtoken](https://github.com/Keats/jsonwebtoken)
* [argon2](https://github.com/RustCrypto/password-hashes/tree/master/argon2)
* [reqwest](https://github.com/seanmonstar/reqwest)
* [env_logger](https://github.com/env-logger-rs/env_logger/)
* [chrono](https://github.com/chronotope/chrono)

## Design

Common functionality not dependent on the web framework is implemented in `backend/common` crate. Common functionality that is dependent on the web framework as well as the ORM models are defined in the `backend/common_web` crate.

At this point of time only support for PostgresSQL is added to the backend, though support for other popular databases can be easily added.

All crates are placed in a single rust workspace to build all binaries at once and to synchronize dependencies. This also helped improve build times via docker. (~5min)

The feed microservice interacts with the DB and an S3 bucket where it stores the uploaded media. Signed URLs are utilized where applicable to reduce overall latency.

The users microservice uses JWTs to authenticate and authorize users. It stores the user information into database. Passwords are salted and hashed using argon2.

The imgproc microservice listens for S3 events from a configured AWS SQS queue. On object creation, it downloads the media from the S3 bucket, generates a thumbnail, and publishes the thumbnail to a separate S3 bucket. On object deletion, it removes the corresponding thumbnail from the thumbnail S3 bucket.

## Building the application

It is possible to build the application locally via docker-compose:

```bash
docker-compose -f ./docker/docker-compose-build-phase1.yaml build --parallel && \
docker-compose -f ./docker/docker-compose-build-phase2.yaml build --parallel
```

## Configuration

You will need to modify `deploy/secrets.yaml` and update those fields accordingly.

To test locally a `.env` file with the following variables should be defined:

```bash
# The database ip address or domain name
POSTGRESS_HOST=
# The name of the database
POSTGRESS_DATABASE=
# The database username and password
POSTGRESS_USERNAME=
POSTGRESS_PASSWORD=
# The AWS region of the cluster and DB
AWS_REGION=
# The local aws profile to use for credentials
AWS_PROFILE=
# The name of the S3 Bucket for storing media
AWS_MEDIA_BUCKET=
# The name of the S3 Bucket for storing thumbnails
AWS_THUMBNAILS_BUCKET=
# The base url of the thumbnails S3 bucket
AWS_THUMBNAILS_BASE_URL=
# The name of the SQS queue where the media bucket sends events
AWS_SQS_QUEUE=
# A random string
JWT_SECRET=
```

## Deploying locally

```bash
docker-compose -f docker/docker-compose.yaml up 
```

## CI/CD

Continuous integration is provided by Github Actions. It is configured to run the very same docker-compose build yamls, but additionally it tags and publishes the images to docker hub. The publishing is only triggered when a version `git tag` is pushed to the repository (any tag starting with `v`). However, builds are performed on every publish to the `main` branch.

## Deployment

The yamls for kubernetes are found in `deploy` folder. These are currently configured to always grab the latest images.
