FROM pandoc/core:latest

COPY run.sh /usr/local/bin/atom-eval-pandoc
RUN chmod +x /usr/local/bin/atom-eval-pandoc

ENTRYPOINT ["/usr/local/bin/atom-eval-pandoc"]
