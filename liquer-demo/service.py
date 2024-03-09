from multiprocessing import freeze_support
import sys
sys.path = ["/Users/orest/OneDrive/Documents/liquer"] + sys.path

# Flask-related imports
import logging
import webbrowser
from flask import Flask, redirect
from liquer.config import load_config, initialize, preset, config
from liquer.store import web_mount, FileStore, get_store
from liquer.state import set_var
from pathlib import Path
import liquer.server.blueprint as bp

app = Flask(__name__)


@app.route('/', methods=['GET', 'POST'])
@app.route('/index.html', methods=['GET', 'POST'])
def index():
    print(f"Redirect to index link: {index_link}")
    return redirect(index_link, code=302)

root = Path(__file__).parent
(root/"gallery").mkdir(exist_ok=True)
web_mount("gallery", FileStore(root/"gallery"))

load_config(root/"config.yaml")
initialize()

url_prefix=config()["setup"].get("url_prefix", "/liquer")
index_link=config()["setup"].get("index_link", "/liquer/web/gui")
set_var("api_path", url_prefix + "/q/")
set_var("server", f"https://orest3d.pythonanywhere.com")

# CORS - used to support e.g. integration of Godot web-applications 
@bp.app.after_request
def add_cross_origin_header(response):
    response.headers['Access-Control-Allow-Origin'] = '*'
    response.headers['Cross-Origin-Embedder-Policy'] = 'require-corp'
    response.headers['Cross-Origin-Opener-Policy'] = 'same-origin'

    return response


app.register_blueprint(bp.app, url_prefix=url_prefix)


#if __name__ == '__main__':
#    freeze_support()
#    load_config(root/"config.yaml")
#    initialize()
#    webbrowser.open("http://localhost:5000")
#    preset().start_server(config())
