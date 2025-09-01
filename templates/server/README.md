<div  align="center" style="margin-top: 3%">
   <h1>
     {{project_name}}
   </h1>
   <h3>
    {{project_description}}
   </h3>
</div>


## Setting up a Virtual Environment


## Quick Start
> Set up a virtual environment via [uv](https://docs.astral.sh/uv)
1. Clone the code
```shell
git clone https://github.com/{{username}}/{{project_name}}.git
cd {{project_name}}
```
2. Download dependencies with [uv](https://docs.astral.sh/uv)
```shell
uv sync
```
3. Database migration

   Modify the **sqlalchemy.url** under **alembic.ini** to your own database connection address.
```shell
uv run alembic upgrade head
```
4. Start the server
```shell
uv run main.py
```
5. Interactive documentation address: http://127.0.0.1:13000/docs
6. You can stop the server at any time by pressing CTRL+C.

## License

[MIT](https://opensource.org/licenses/MIT).
