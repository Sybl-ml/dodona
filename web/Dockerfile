FROM node:8

WORKDIR /usr/src/app

RUN yarn run build

CMD ["serve", "-p", "3000", "-s", "."]