import tornado.ioloop
import tornado.web
from liquer import *
import json
from liquer.commands import command_registry
from liquer.state_types import encode_state_data, state_types_registry


import liquer.ext.basic
import liquer.ext.meta
import liquer.ext.lq_pandas
import liquer.ext.lq_hxl
import liquer.ext.lq_python
import liquer.ext.lq_pygments


def liquer_static_path():
    import liquer
    import os.path
    return os.path.join(os.path.dirname(liquer.__file__),"static")


class MainHandler(tornado.web.RequestHandler):
    def get(self):
        self.write("Hello, world")

class LiquerIndexHandler(tornado.web.RequestHandler):
    def get(self):
        self.redirect("/static/index.html")

class LiquerIndexJsHandler(tornado.web.RequestHandler):
    def get(self):
        self.redirect("/static/index.js")

#@app.route('/api/commands.json')
class CommandsHandler(tornado.web.RequestHandler):
    def get(self):
        """Returns a list of commands in json format"""
        self.write(json.dumps(command_registry().as_dict()))

#@app.route('/api/debug-json/<path:query>')
class DebugQueryHandler(tornado.web.RequestHandler):
    def prepare(self):
        header = "Content-Type"
        body = "application/json"
        self.set_header(header, body)

    def get(self, query):
        """Debug query - returns metadata from a state after a query is evaluated"""
        state = evaluate(query)
        state_json = state.as_dict()
        self.write(json.dumps(state_json))

#@app.route('/q/<path:query>')
class QueryHandler(tornado.web.RequestHandler):
    def get(self, query):
        """Main service for evaluating queries"""
        state = evaluate(query)
        filename = state.filename
        extension = None
        if filename is not None:
            if "." in filename:
                extension = filename.split(".")[-1]

        b, mimetype, type_identifier = encode_state_data(
            state.get(), extension=extension)
        if filename is None:
            filename = state_types_registry().get(type_identifier).default_filename()

        header = "Content-Type"
        body = mimetype
        self.set_header(header, body)

        self.write(b)


if __name__ == "__main__":
    application = tornado.web.Application([
#        (r"/", MainHandler),
        (r"/api/commands.json", CommandsHandler),
        (r"/api/debug-json/(.*)", DebugQueryHandler),
        (r"/q/(.*)", QueryHandler),
        (r"/liquer/q/(.*)", QueryHandler),
        (r'/static/(.*)', tornado.web.StaticFileHandler, {'path': liquer_static_path()}),
        (r'/', LiquerIndexHandler),
        (r'/index.html', LiquerIndexHandler),
        (r'/index.js', LiquerIndexJsHandler),
    ])
    application.listen(8888)
    tornado.ioloop.IOLoop.current().start()
