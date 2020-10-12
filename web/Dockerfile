FROM node:11.1-alpine

WORKDIR usr/src/app

COPY package*.json ./
RUN yarn install
RUN yarn run

EXPOSE 8080