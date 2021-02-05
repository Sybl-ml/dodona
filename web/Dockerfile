FROM node:11.1-alpine

WORKDIR /usr/src/app

RUN npm install -g yarn
ADD package.json yarn.lock ./
RUN yarn upgrade
RUN yarn install && yarn cache clean

EXPOSE 8080