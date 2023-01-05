FROM node:19-alpine
COPY supervisord.conf /etc/supervisord.conf
COPY nginx.conf /etc/nginx/nginx.conf
COPY package.json /app/package.json
COPY yarn.lock /app/yarn.lock
COPY index.js /app/index.js
RUN apk add --no-cache \
		supervisor \
		nginx \
	&& cd app && yarn install
HEALTHCHECK --interval=60s --retries=5 CMD curl --fail http://localhost/ || exit 1
CMD ["/usr/bin/supervisord", "-n", "-c", "/etc/supervisord.conf"]