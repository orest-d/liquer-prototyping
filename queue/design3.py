# Simple tornado server example

import asyncio
from enum import Enum
from multiprocessing.managers import BaseManager
import random
from typing import Any
import webbrowser
import tornado
import tornado.web
import multiprocessing
import concurrent.futures
import time
from dataclasses import dataclass


class IndexHandler(tornado.web.RequestHandler):
    async def get(self):
        self.write(
            """<html>
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
</html>"""
        )


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


JobManager.register("create_job_queue", callable=lambda: MasterJobQueue())
JobManager.register("create_worker_registry", callable=lambda n=2: WorkerRegistry(n))

class JobStatus(Enum):
    """Job status in a job queue"""

    UNKNOWN = "unknown"  # Unknown status
    NOT_IN_QUEUE = "not in queue"  # Job is not in the queue
    QUEUED = "queued"  # Job is in the queue, but not assigned to a worker
    ASSIGNED = "assigned"  # Job is assigned to a worker, but not started yet
    RUNNING = "running"  # Job is running
    WAITING = "waiting"  # Job is waiting for a dependency
    COMPLETED = "completed"  # Job is completed
    FAILED = "failed"  # Job failed

    def is_done(self):
        """Check if the job is done"""
        return self == JobStatus.COMPLETED or self == JobStatus.FAILED

# data class with worker info:
# - worker id
# - worker status
# - start time
# - last ping time
# - worker channel
# - worker process
# Worker info is stored in the WorkerRegistry


@dataclass
class WorkerInfo:
    worker_id: str
    start_time: float
    last_update_time: float
    channel: multiprocessing.connection.Connection
    process: multiprocessing.Process

@dataclass
class JobInfo:
    query: str
    worker_id: str
    start_time: float
    last_update_time: float
    status: JobStatus = JobStatus.UNKNOWN
    result: Any = None
    error: Any = None
    dependency: str = None
    message: str = None

    @classmethod
    def from_query(cls, query, status=JobStatus.UNKNOWN):
        return cls(
            query=query,
            status=status,
            start_time=time.time(),
            last_update_time=time.time(),
            result=None,
            error=None,
            worker_id=None,
            dependency=None,
            message=None,
        )
    
    def ping(self):
        self.last_update_time = time.time()
        return self
    
    def queued(self):
        self.status = JobStatus.QUEUED
        return self.ping()

    def assign_to(self, worker_id):
        assert self.status == JobStatus.QUEUED
        self.status = JobStatus.ASSIGNED
        self.worker_id = worker_id
        return self.ping()
    
    def running(self):
        assert (self.status == JobStatus.ASSIGNED or self.status == JobStatus.WAITING) and self.worker_id is not None
        self.status = JobStatus.RUNNING
        return self.ping()

    def waiting(self, dependency):
        assert self.status == JobStatus.QUEUED or self.status == JobStatus.ASSIGNED or self.status == JobStatus.RUNNING
        self.status = JobStatus.WAITING
        self.dependency = dependency
        return self.ping()
    
    def completed(self, result):
        print(f"Changing Job {self.query} to completed")
        print(f"  Job {self.query} status is {self.status}")
        assert self.status == JobStatus.RUNNING
        self.status = JobStatus.COMPLETED
        self.result = result
        self.error = None
        return self.ping()
    
    def failed(self, error):
        assert self.status == JobStatus.RUNNING
        self.status = JobStatus.FAILED
        self.error = error
        self.result = None
        return self.ping()
    
    def is_done(self):
        return self.status.is_done()
    
class WorkerRegistry:
    def __init__(self, num_workers):
        self.num_workers = num_workers
        self.workers = {}
        self.id_num = 0

    def get(self, worker_id):
        return self.workers[worker_id]

    def register_worker(self, worker_id, channel):
        self.workers[worker_id] = WorkerInfo(
            worker_id=worker_id,
            start_time=time.time(),
            last_update_time=time.time(),
            channel=channel,
            process=None,
        )       

    def new_worker_id(self):
        """Returns a new identifier"""
        self.id_num += 1
        return f"Worker_{self.id_num}"

    def new_worker_old(self, jq):
        """Create and starts one new worker"""
        (parent_channel, child_channel) = multiprocessing.Pipe()
        worker_id = self.new_worker_id()
        p = multiprocessing.Process(
            target=worker_process_old, args=(jq, worker_id, child_channel)
        )
        p.start()
        # jq.register_channel(worker_id, parent_channel)
        self.workers[worker_id] = WorkerInfo(
            worker_id=worker_id,
            start_time=time.time(),
            last_update_time=time.time(),
            channel=parent_channel,
            process=p,
        )

    def new_worker(self):
        """Create and starts one new worker"""
        (parent_channel, child_channel) = multiprocessing.Pipe()
        worker_id = self.new_worker_id()
        p = multiprocessing.Process(
            target=worker_process, args=(worker_id, child_channel)
        )
        p.start()
        # jq.register_channel(worker_id, parent_channel)
        self.workers[worker_id] = WorkerInfo(
            worker_id=worker_id,
            start_time=time.time(),
            last_update_time=time.time(),
            channel=parent_channel,
            process=p,
        )

    def start_workers(self):
        """Starts the workers"""
        while len(self.workers)<self.num_workers:
            self.new_worker()

        

class JobRegistry(object):
    def __init__(self):
        self.jobs = {}
    
    def contains(self, query):
        return query in self.jobs

    def get(self, query):
        return self.jobs[query]
    
    def set(self, info):
        self.jobs[info.query] = info

    def add(self, query, status=JobStatus.UNKNOWN):
        if not self.contains(query):
            self.jobs[query] = JobInfo.from_query(query, status=status)
        return self.get(query)
    
    def status(self, query):
        if self.contains(query):
            return self.get(query).status
        else:
            return JobStatus.NOT_IN_QUEUE
        
    def __getitem__(self, query):
        return self.jobs[query]
    
    def __len__(self):
        return len(self.jobs)

def execute_job(jq, job):
    print(f"Executing {job}")
    try:
        i = int(job[3:])
    except:
        i = 0

    if i:
        dependency_job = f"Job{i-1}"
        print(f"{job} depends on {dependency_job}")
        jq.submit(dependency_job)
    time.sleep(random.random() * 0.1)
    #    if type(dependency) == str:
    #        dependency_value = dependency
    #    else:
    #        dependency_value = dependency.get()
    dependency_value = "-not available-"
    if i:
        dependency_value = str(jq.wait_for(dependency_job).result)
        print(f"Dependency {dependency_job} received {dependency_value}")
    else:
        print(f"No Dependency for {job}")
        dependency_value = "~"

    print(f"Returning result for {job}")
    return "Result-" + job + "(" + dependency_value + ")"


class WorkerJobQueue:
    def __init__(self, worker_id, master_job_queue, channel):
        self.worker_id = worker_id
        self.master_job_queue = master_job_queue
        self.channel = channel
        self.local_jobs = JobRegistry()
        self.current_job=None

    def get_job(self):
        """Returns the next job for the worker"""
        self.current_job = self.master_job_queue.get_job_for(self.worker_id)
        return self.current_job
    
    def get_status(self, query):
        """Returns the status of the given job"""
        if self.local_jobs.contains(query):
            return self.local_jobs.status(query)
        return self.master_job_queue.get_status(query)
    
    def submit(self, query):
        """Submits a new job to the queue.
        Worker will be notified when the job is done.
        """
        if self.local_jobs.contains(query):
            return False
        else:
            self.local_jobs.add(query, JobStatus.UNKNOWN)
        self.master_job_queue.worker_request(self.worker_id, query)
        return True

    def get_result(self, query):
        """Returns the result of the given job"""
        if self.local_jobs.contains(query):
            return self.local_jobs[query].result
        return self.master_job_queue.get_result(query)

    def set_result(self, query, result):
        """Sets the result of the given job"""
        print(f"WorkerJobQueue {self.worker_id} received result {result} for {query}")
        if not self.local_jobs.contains(query):
            self.local_jobs.add(query, JobStatus.RUNNING)
        self.local_jobs[query].completed(result)
        self.master_job_queue.set_result(query, result)

    def get_error(self, query):
        """Returns the error of the given job"""
        if self.local_jobs.contains(query):
            return self.local_jobs[query].error
        return self.master_job_queue.get_error(query)

    def set_error(self, query, error):
        """Sets the error of the given job"""
        self.local_jobs[query].failed(error)
        self.master_job_queue.set_error(query, error)

    def receive_one_event(self):
        info = self.channel.recv()
        self.local_jobs.set(info)
        return info
    
    def wait_for(self, query):
        """Waits until the given job is done"""
        self.master_job_queue.set_waiting(self.current_job, query)
        if not self.local_jobs.contains(query):
            self.submit(query)

        while self.local_jobs[query].is_done() is False:
            self.receive_one_event()
        self.master_job_queue.set_running(self.current_job)

        return self.local_jobs[query]

def worker_process_old(jq, worker_id, channel):
    wjq = WorkerJobQueue(worker_id, jq, channel)
    while True:
        job = wjq.get_job()
        if job is None:
            print(f"None job, Broken queue? worker: {worker_id}")
            continue
        print(f"Worker {worker_id} got job {job}")
        jq.set_running(job)
        result = execute_job(wjq, job)
        wjq.set_result(job, result)
        print(f"Worker {worker_id} finished job {job}")

def worker_process(worker_id, channel):
    print(f"Worker {worker_id} started")
    jq = channel.recv()
    print(f"Worker {worker_id} obtained job queue")
    wjq = WorkerJobQueue(worker_id, jq, channel)
    while True:
        job = wjq.get_job()
        if job is None:
            print(f"None job, Broken queue? worker: {worker_id}")
            continue
        print(f"Worker {worker_id} got job {job}")
        jq.set_running(job)
        result = execute_job(wjq, job)
        wjq.set_result(job, result)
        print(f"Worker {worker_id} finished job {job}")


class MasterJobQueue:
    def __init__(self, number_of_workers=4):
        """Initializes the job queue with the given number of workers"""
        self.queue = multiprocessing.Queue()
        self.workers = WorkerRegistry(number_of_workers)
        self.jobs = JobRegistry()
        self.requests = {}
        self.callbacks = {}

    def start_workers(self):
        self.workers.start_workers()
    
    def channels(self):
        ch=[]
        for w in self.workers.workers.values():
            print(w)
            ch.append(w.channel)
        return ch
    
#    def set_workers(self, workers):
#        """Sets the worker registry"""
#        self.workers = workers

#    def register_worker(self, worker_id, channel):
#        """Registers a new worker"""
#        self.workers.register_worker(worker_id, channel)

    def get_job_for(self, worker_id):
        """Returns the next job"""
        query = self.queue.get()
        print(f"MasterJobQueue: Worker {worker_id} is getting job {query}")
        self.jobs[query].assign_to(worker_id)
        return query
    
    def assign_to(self, worker_id, query):
        """Assigns the given job to the given worker"""
        if not self.jobs.contains(query):
            self.submit(query)
        info = self.jobs[query]
        if info.status == JobStatus.QUEUED:
            info.assign_to(worker_id)
            return True
        return False

    def worker_request(self, worker_id, query):
        """Requests the given job"""

        if not self.jobs.contains(query):
            self.submit(query)

        info = self.jobs.get(query)
        self.workers.get(worker_id).channel.send(info)

        if not info.is_done():
            self.requests[query] = self.requests.get(query, set())
            self.requests[query].add(worker_id)
    
    def get_status(self, query):
        """Returns the status of the given job"""
        return self.jobs[query].status
    
    def set_running(self, query):
        """Sets the given job to running"""
        print(f"MasterJobQueue: Setting {query} to running")
        self.jobs[query].running()
        self.notify(query)

    def set_waiting(self, query, dependency):
        """Sets the given job to waiting for dependency"""
        print(f"MasterJobQueue: Setting {query} to waiting for {dependency}")
        self.jobs[query].waiting(dependency)
        self.notify(query)
    
    def submit(self, query):
        """Submits a new job to the queue"""
        print(f"Submitting {query}")
        if self.jobs.contains(query):
            return False
        self.jobs.add(query, JobStatus.QUEUED)
        self.queue.put(query)        
        return True

    def add_callback(self, query, callback):
        """Adds a callback to the given job"""
        if not self.jobs.contains(query):
            self.submit(query)
        if query not in self.callbacks:
            self.callbacks[query] = []
        self.callbacks[query].append(callback)

    def notify(self, query):
        print(f"MasterJobQueue: Notifying {query}")
        info = self.jobs.get(query)
        print(f"  - Status {info.status}")
        if info.is_done():
            print(f"  - Done {query}")
            for worker_id in self.requests.get(query, set()):
                print(f"  - Notifying {worker_id} about {query} status {info.status}")
                self.workers.get(worker_id).channel.send(info)
            if query in self.callbacks:
                for callback in self.callbacks[query]:
                    print(f"  - Calling {callback}")
                    callback(info)
                print(f"  - removing callbacks for {query}")
                del self.callbacks[query]

    def get_result(self, job):
        """Returns the result of the given job"""
        return self.jobs.get(job).result

    def set_result(self, query, result):
        """Sets the result of the given job"""
        print(f"MasterJobQueue received result {result} for {query}")
        if not self.jobs.contains(query):
            print(f"MasterJobQueue: Job {query} not found")
        else:
            print(f"MasterJobQueue: Job {query} will be labeled as completed")
            self.jobs[query].completed(result)
        self.notify(query)

    def get_error(self, query):
        """Returns the error of the given job"""
        return self.jobs.get(query).error

    def set_error(self, query, error):
        """Sets the error of the given job"""
        self.jobs[query].failed(error)
        self.notify(query)

##########################################################################################


def make_app():
    return tornado.web.Application(
        [
            (r"/", IndexHandler),
            (r"/test1a.txt", ProcessHandler1a),
            (r"/test1b.txt", ProcessHandler1b),
            (r"/test2a.txt", ProcessHandler2a),
            (r"/test2b.txt", ProcessHandler2a),
        ]
    )


async def main():
    app = make_app()
    app.listen(8888)
    await asyncio.Event().wait()


_executor = None


def get_executor():
    global _executor
    if _executor is None:
        _executor = concurrent.futures.ProcessPoolExecutor()
    return _executor


#if __name__ == "__main__":
#    webbrowser.open("http://localhost:8888")
#    # asyncio.run(main())
#
#    app = make_app()
#    app.listen(8888)
#
#    tornado.ioloop.IOLoop.current().start()

if __name__ == "__main__":
    with JobManager() as manager:
#        workers = manager.create_worker_registry()
#        workers = WorkerRegistry(2)
        jq = manager.create_job_queue()
        jq.start_workers()
        channels = jq.channels()
        print(f"Channels: {channels}")
        for ch in channels:
            ch.send(jq)

#        workers.start_workers()
#        jq.set_workers(workers)

        jq.submit("Job2")
        input("Press enter to continue")
