from concurrent.futures import Future
from enum import Enum
import multiprocessing


class JobStatus(Enum):
    """Job status in a job queue"""
    UNKNOWN = 0  # Unknown status
    NOT_IN_QUEUE = 1  # Job is not in the queue
    QUEUED = 2  # Job is in the queue, but not assigned to a worker
    ASSIGNED = 3  # Job is assigned to a worker, but not started yet
    RUNNING = 4  # Job is running
    COMPLETED = 5  # Job is completed
    FAILED = 6  # Job failed

class WorkerAccess:
    """Queue interface for worker objects"""
    def pop_job(self, worker_id: str):
        """Pops a job from the queue"""
        pass

    def submit_assigned(self, query:str, worker_id: str)->bool:
        """Submits a job and assign it to a worker"""
        pass

    def finished_job(self, query: str, result: object, worker_id: str):
        """Marks a job as finished"""
        pass

    def failed_job(self, query: str, error:str, worker_id: str):
        """Marks a job as failed"""
        pass

class Worker(object):
    def __init__(self, job_queue:WorkerAccess, worker_id: str):
        self.worker_id = worker_id
        self.job_queue = job_queue
        self.local_queue = []
        self.running_job = None

    def identifier(self):
        """Returns the worker identifier"""
        return self.worker_id

    def start(self):
        """Starts the worker"""
        pass

    def stop(self):
        """Stops the worker"""
        pass

    def submit(self, query: str) -> None:
        """Submits a job to the worker"""
        self.local_queue.append(query)
        return None

    def status(self, query: str) -> JobStatus:
        """Returns the status of a job"""
        if query in self.local_queue:
            return JobStatus.QUEUED
        elif query == self.running_job:
            return JobStatus.RUNNING
        else:
            return JobStatus.NOT_IN_QUEUE

    def run(self):
        """Runs the worker"""
        pass

def worker_process(queue, worker_id: str):
    """Worker process"""
    print(f"Start worker {worker_id}")
    
class JobQueue(object):
    def __init__(self, manager=None, parameters=None, num_workers: int = 4):
        """Initializes the job queue with the given number of workers"""
        self.num_workers = num_workers
        self.worker_queues = {}
        self.worker_processes = {}
        self.results = {}
        self.errors = {}
        self.id_num=0
        if manager is None:
            self.manager = multiprocessing.Manager()
        else:
            self.manager = manager
        self.parameters == parameters
        self.queue = self.manager.Queue()

    def new_identifier(self):
        """Returns a new identifier"""
        self.id_num += 1
        return f"Worker_{self.id_num}"
    
    def spawn_worker(self):
        """Spawns a new worker"""
        queue = self.manager.Queue()
        worker_id = self.new_identifier()
        parameters = self.parameters

        pass

    def submit(self, query: str) -> None:
        """Submit a job to the queue, returns immediately"""
        pass

    def status(self, query: str) -> JobStatus:
        """Returns the status of a job"""
        return JobStatus.UNKNOWN

    def result(self, query: str):
        """Returns the result of a completed job
        or raises an exception if the job failed
        or is not completed yet
        """
        if query in self.results:
            return self.results[query]
        elif query in self.errors:
            raise Exception(f"Job '{query}' failed with error:\n"+self.errors[query])
        else:
            raise Exception(f"Result of '{query}' not available")
        
    def cancel(self, query: str) -> None:
        """Cancels a job"""
        return None

    def get(self, query: str) -> Future:
        """Returns a Future object if job is in the queue, None otherwise"""
        return None

    def evaluate(self, query: str) -> None:
        """Evaluates a job, returns a future object"""
        self.submit(query)
        f = self.get(query)
        if f is None:
            raise Exception(f"Job submission of '{query}' failed")
        return f
    