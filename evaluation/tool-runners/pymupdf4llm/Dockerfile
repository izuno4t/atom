FROM python:3.12-slim

RUN pip install --no-cache-dir pymupdf4llm

COPY run.py /opt/atom-eval/run.py

ENTRYPOINT ["python", "/opt/atom-eval/run.py"]
