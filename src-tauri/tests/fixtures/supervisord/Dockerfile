FROM python:3.11-slim
RUN pip install --no-cache-dir setuptools supervisor==4.2.5
COPY supervisord.conf /etc/supervisord.conf
CMD ["supervisord", "-c", "/etc/supervisord.conf"]
