# Simple tornado server example

import asyncio
from enum import Enum
from multiprocessing.managers import BaseManager, SyncManager
import random
import traceback
from typing import Any
import webbrowser
import multiprocessing
import concurrent.futures
import time
from dataclasses import dataclass


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

class WorkerStatus(Enum):
    SPAWNED = "spawned"
    STARTING = "starting"
    READY = "ready"
    BUSY = "busy"

@dataclass
class WorkerInfo:
    worker_id: str
    start_time: float
    last_update_time: float
    channel: multiprocessing.connection.Connection
    process: multiprocessing.Process
    worker_status: WorkerStatus = WorkerStatus.SPAWNED


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
        assert (
            self.status == JobStatus.ASSIGNED
            or self.status == JobStatus.WAITING
            or self.status == JobStatus.RUNNING
        ) and self.worker_id is not None
        self.status = JobStatus.RUNNING
        return self.ping()

    def waiting(self, dependency):
        assert (
            self.status == JobStatus.QUEUED
            or self.status == JobStatus.ASSIGNED
            or self.status == JobStatus.RUNNING
            or self.status == JobStatus.WAITING
        )
        self.status = JobStatus.WAITING
        self.dependency = dependency
        return self.ping()

    def completed(self, result):
        print(f"Changing Job {self.query} to completed")
        print(f"  Job {self.query} status is {self.status}")
#        assert self.status == JobStatus.RUNNING
        self.status = JobStatus.COMPLETED
        self.result = result
        self.error = None
        return self.ping()

    def failed(self, error):
        # assert self.status == JobStatus.RUNNING
        self.status = JobStatus.FAILED
        self.error = error
        self.result = None
        return self.ping()

    def is_done(self):
        return self.status.is_done()

class Message(object):
    def __init__(self, worker_id):
        self.worker_id = worker_id
    def __str__(self):
        return f"{self.__class__.__name__}({self.worker_id})"

class QueryMessage(Message):
    def __init__(self, worker_id, query):
        super().__init__(worker_id)
        self.query = query
    def __str__(self):
        return f"{self.__class__.__name__}({self.worker_id}, {self.query})"

class Ping(Message):pass
class Pong(Message):pass
class WorkerStarting(Message):pass
class WorkerReady(Message):pass
class WorkerWrongRequest(Message):pass

class SubmitJob(QueryMessage):pass
class WorkerAcceptedJob(QueryMessage):pass
class WorkerRejectedJob(QueryMessage):pass
class WorkerFinishedJob(Message):pass
class WorkerFailedJob(Message):pass
class WorkerMessage(Message):
    def __init__(self, worker_id, query, message):
        super().__init__(worker_id)
        self.message = message
    def __str__(self):
        return f"{self.__class__.__name__}({self.worker_id}, {self.query}, {self.message})"

class WorkerInterestedInJob(Message):pass
class JobResultReady(Message):pass
class JobResultPending(Message):pass
class JobAssigned(Message):pass

class Worker:
    """Worker"""
    def __init__(self, worker_id, channel):
        self.worker_id = worker_id
        self.channel = channel
        self.queue = []

    def process_message(self, message):
        if isinstance(message, Ping):
            return self.process_ping()
        elif isinstance(message, SubmitJob):
            return self.process_submit_job(message.query)
        else:
            return self.channel.send(WorkerWrongRequest(self.worker_id, message))
        
    def process_ping(self):
        return self.channel.send(Pong(self.worker_id))

    def process_submit_job(self, query):
        if len(self.queue) > 0:
            return self.channel.send(WorkerRejectedJob(self.worker_id, query))
        self.channel.send(WorkerAcceptedJob(self.worker_id, query))
        self.queue.append(query)
        self.execute_jobs()

    def execute_jobs(self):
        while len(self.queue) > 0:
            query = self.queue[0]
            try:
                result = self.execute_query(query)
                self.channel.send(WorkerFinishedJob(self.worker_id, query, result))
                self.queue.pop(0)
            except:
                e = traceback.format_exc()
                self.channel.send(WorkerFailedJob(self.worker_id, query, e))
                self.queue.pop(0)

    def execute_query(self, query):
        """Execute a query"""
        time.sleep(1)
        return "Result of " + query
    
    def __str__(self):
        return f"Worker {self.worker_id}"
    
class WorkerRegistry:
    """Registry of workers"""
    def __init__(self, num_workers):
        self.num_workers = num_workers
        self.workers = {}
        self.id_num = 0

    def get(self, worker_id):
        return self.workers[worker_id]

    def new_worker_id(self):
        """Returns a new identifier"""
        self.id_num += 1
        return f"Worker_{self.id_num}"

    def new_worker(self):
        """Create and starts one new worker"""
        (parent_channel, child_channel) = multiprocessing.Pipe()
        worker_id = self.new_worker_id()
        p = multiprocessing.Process(
            target=worker_process, args=(worker_id, child_channel)
        )
        p.start()
        self.workers[worker_id] = WorkerInfo(
            worker_id=worker_id,
            start_time=time.time(),
            last_update_time=time.time(),
            channel=parent_channel,
            process=p,
        )

    def start_workers(self):
        """Starts the workers"""
        while len(self.workers) < self.num_workers:
            self.new_worker()

    def stop_workers(self):
        """Stops all workers"""
        for w in self.workers.values():
            if w.process is not None:
                w.process.terminate()
                w.process.join()
        self.workers = {}

    def poll_messages(self):
        """Polls messages from workers"""
        processed=0
        for w in self.workers.values():
            if w.channel.poll():
                message = w.channel.recv()
                processed+=1
                print(f"Received message {message}")
                if isinstance(message, WorkerReady):
                    w.worker_status = WorkerStatus.READY
                    w.last_update_time = time.time()
                elif isinstance(message, WorkerStarting):
                    w.worker_status = WorkerStatus.STARTING
                    w.last_update_time = time.time()
                elif isinstance(message, Pong):
                    w.last_update_time = time.time()
                elif isinstance(message, WorkerAcceptedJob):
                    w.worker_status = WorkerStatus.BUSY
                    w.last_update_time = time.time()
                elif isinstance(message, WorkerRejectedJob):
                    w.worker_status = WorkerStatus.BUSY
                    w.last_update_time = time.time()
                else:
                    print(f"Unknown message {message}")
        return processed

    def submit_job(self, query):
        """Submit a job if there is a free worker"""
        for w in self.workers.values():
            if w.worker_status == WorkerStatus.READY:
                w.channel.send(SubmitJob(w.worker_id, query))
                w.worker_status = WorkerStatus.BUSY
                return True
        return False

def execute_query(query):
    """Execute a query"""
    time.sleep(1)
    return "Result of " + query

def worker_process(worker_id, channel):
    print(f"Worker {worker_id} starting")
    channel.send(WorkerStarting(worker_id))
    worker = Worker(worker_id, channel)
    channel.send(WorkerReady(worker_id))

    while True:
        message = channel.recv()
        worker.process_message(message)

if __name__ == "__main__":
    registry = WorkerRegistry(2)
    registry.start_workers()
    time.sleep(2)
    for w in registry.workers.values():
        w.channel.send(Ping(w.worker_id))
        print(f"Worker {w.worker_id} sent ping")
    time.sleep(2)
    registry.poll_messages()
    registry.poll_messages()
    registry.poll_messages()
#    for w in registry.workers.values():
#        print(f"Worker {w.worker_id} received {w.channel.recv()}")
    registry.stop_workers()
    print("Done")