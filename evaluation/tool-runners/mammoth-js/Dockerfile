FROM node:22-slim

WORKDIR /opt/atom-eval
RUN npm install mammoth turndown

COPY run.mjs /opt/atom-eval/run.mjs

ENTRYPOINT ["node", "/opt/atom-eval/run.mjs"]
