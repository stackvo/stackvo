# Python

A Django, Flask or FastAPI project at `https://api.loc`, with the packages
installed in the container and a PostgreSQL to talk to.

## 1. Create it

**Projects → +**, then **Django**, **Flask** or **FastAPI** under the
templates, or **Empty project** with the **Python** runtime and a version —
2.7 to 3.14. Name it `api`.

For an empty project the form asks for the commands:

| Field | Typical value |
| --- | --- |
| Install command | `pip install -r requirements.txt` |
| Build command | empty, or `python manage.py collectstatic --noinput` |
| Start command | `gunicorn app:app --bind 0.0.0.0:8000`, or `uvicorn main:app --host 0.0.0.0 --port 8000`, or `python manage.py runserver 0.0.0.0:8000` |

The container's port is what the proxy points at; bind to `0.0.0.0`, not to
`127.0.0.1`, or the proxy cannot reach it.

## 2. A database

**Catalogue → Available** installs PostgreSQL; **Project → Services this
project needs** declares it for the project and shows the host, the port and
the password. Put them in the project's settings or `.env`, then migrate from
the project's folder:

```bash
stackvo python manage.py migrate
```

## 3. Packages

`stackvo python`, `stackvo exec pip` and `stackvo shell` run inside the container,
on the container's Python. A package added to `requirements.txt` is installed
on the next rebuild; for a quick try, install it from `stackvo shell` and
add it to the file when it stays.

## 4. Working on the code

Editing a file on the host changes nothing in the container until a rebuild,
because the image carries a copy taken at build time. For an edit-and-reload
loop, use the framework's own reloader — `runserver`, `flask --debug`,
`uvicorn --reload` — as the start command and mount the source with
**Project → Dev server** switched on, the same switch Node projects use.

## Every day

```bash
stackvo python -V
stackvo exec pip list
stackvo python manage.py createsuperuser
stackvo shell
```
