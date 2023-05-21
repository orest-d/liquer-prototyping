# Simple tornado server example

import asyncio
from multiprocessing.managers import BaseManager
from typing import Any
import webbrowser
import tornado
import tornado.web
import multiprocessing
import concurrent.futures

class IndexHandler(tornado.web.RequestHandler):
    async def get(self):
        self.write("""<html>
<head><title>Test</title></head>
<body>
<h1>Test</h1>
<ul>
<li><a href="/">Home</a></li>
<li><a href="/test1a.txt" target="_blank">Test 1a</a></li>
<li><a href="/test1b.txt" target="_blank">Test 1b</a></li>
<li><a href="/test2a.txt" target="_blank">Test 2a</a></li>
<li><a href="/test2b.txt" target="_blank">Test 2b</a></li>
</ul>
</body>
</html>""")

def worker_process1(result):
    import time
    print("Worker process started")
    time.sleep(5)
    result.put("Hello World - process")
    print("Worker process finished")

def worker_function1():
    import time
    print("Worker function started")
    time.sleep(5)
    print("Worker function finished")
    return "Hello World - function"

def as_tornado_future(future):
    tornado_future = asyncio.get_event_loop().create_future()
    def callback(future):
        tornado_future.set_result(future.result())

    tornado.ioloop.IOLoop.current().add_future(future, callback)
    return tornado_future

class ProcessHandler1a(tornado.web.RequestHandler):
    async def get(self):
        self.write("""Start 1A\n""")
        result = multiprocessing.Queue()
        multiprocessing.Process(target=worker_process1, args=(result,)).start()
        self.write(f"""Result 1A: {result.get()}\n""")
class ProcessHandler1b(tornado.web.RequestHandler):
    async def get(self):
        self.write("""Start 1B\n""")
        result = multiprocessing.Queue()
        multiprocessing.Process(target=worker_process1, args=(result,)).start()
        self.write(f"""Result 1B: {result.get()}\n""")
class ProcessHandler2a(tornado.web.RequestHandler):
    async def get(self):
        self.write("""Start 2A\n""")
        executor = get_executor()
        result = await as_tornado_future(executor.submit(worker_function1))

        self.write(f"""Result 2A: {result}\n""")
##########################################################################################

class JobManager(BaseManager):
    pass

JobManager.register('create_job_queue', callable=lambda: JobQueue())

class JobStatus(Enum):
    """Job status in a job queue"""
    UNKNOWN = 0  # Unknown status
    NOT_IN_QUEUE = 1  # Job is not in the queue
    QUEUED = 2  # Job is in the queue, but not assigned to a worker
    ASSIGNED = 3  # Job is assigned to a worker, but not started yet
    RUNNING = 4  # Job is running
    COMPLETED = 5  # Job is completed
    FAILED = 6  # Job failed


class JobQueue:
    def __init__(self, parameters=None, num_workers: int = 4):
        """Initializes the job queue with the given number of workers"""
        self.num_workers = num_workers
        self.worker_processes = {}
        self.results = {}
        self.errors = {}
        self.status = {}
        self.id_num=0
        self.parameters = parameters
        self.queue = multiprocessing.Queue()
        self.callbacks={}

    @staticmethod
    def worker_processes(job_queue_proxy, worker_id, parameters):
        print(f"Worker {worker_id} started")

        while True:
            job = job_queue_proxy.get_job()
            if job is None:
                print(f"Worker {worker_id} killed")                
                break

            print(f"Worker {worker_id} got job {job}")
            job_queue_proxy.set_status(job, JobStatus.ASSIGNED, worker_id)
            try:
                job_queue_proxy.set_status(job, JobStatus.RUNNING, worker_id)
                result = f"Result of job {job}"
                job_queue_proxy.set_result(job, result)
            except Exception as e:
                job_queue_proxy.set_error(job, e)

    def new_identifier(self):
        """Returns a new identifier"""
        self.id_num += 1
        return f"Worker_{self.id_num}"
    

    def spawn_worker(self, worker_id=None):
        """Spawns a worker process"""
        if worker_id is None:
            worker_id = self.new_identifier()

        process = multiprocessing.Process(target=self.worker_processes, args=(self, worker_id, self.parameters))
        self.worker_processes[worker_id] = process
        process.start()

    def start_workers(self):
        """Starts the worker processes"""
        while len(self.worker_processes) < self.num_workers:
            self.spawn_worker()

    def stop_workers(self):
        """Stops the worker processes"""
        for process in self.worker_processes.values():
            process.terminate()
        self.worker_processes = {}

    def get_job(self):
        """Returns the next job"""
        return self.queue.get()
    
    def set_status(self, job, status, arg=None):
        """Sets the status of the given job"""
        self.status[job] = (status, arg)

    def get_status(self, job):
        """Returns the status of the given job"""
        return self.status.get(job, (JobStatus.UNKNOWN, None))[0]

    def submit_job(self, job):
        """Submits a new job to the queue"""
        if self.get_status(job) == JobStatus.NOT_IN_QUEUE:
            self.queue.put(job)
            self.set_status(job, JobStatus.QUEUED)
            return JobStatus.QUEUED
        else:
            return self.get_status(job)
        
    def get_result(self, job):
        """Returns the result of the given job"""
        return self.results.get(job, None)
    
    def set_result(self, job, result):
        """Sets the result of the given job"""
        self.results[job] = result
        self.set_status(job, JobStatus.COMPLETED)

    def get_error(self, job):
        """Returns the error of the given job"""
        return self.errors.get(job, None)
    
    def set_error(self, job, error):
        """Sets the error of the given job"""
        self.errors[job] = error
        self.set_status(job, JobStatus.FAILED)

    def add_callback(self, job, callback):
        """Adds a callback to the given job"""
        pass


##########################################################################################

def make_app():
    return tornado.web.Application([
        (r"/", IndexHandler),
        (r"/test1a.txt", ProcessHandler1a),
        (r"/test1b.txt", ProcessHandler1b),
        (r"/test2a.txt", ProcessHandler2a),
        (r"/test2b.txt", ProcessHandler2a),
    ])

async def main():
    app = make_app()
    app.listen(8888)
    await asyncio.Event().wait()

_executor=None
def get_executor():
    global _executor
    if _executor is None:
        _executor = concurrent.futures.ProcessPoolExecutor()
    return _executor

if __name__ == "__main__":
    webbrowser.open("http://localhost:8888")
    #asyncio.run(main())

    app = make_app()
    app.listen(8888)

    tornado.ioloop.IOLoop.current().start()

