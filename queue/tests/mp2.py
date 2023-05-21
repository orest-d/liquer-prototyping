import multiprocessing as mp
from multiprocessing.managers import BaseManager, SyncManager, Namespace
from os import getpid
from time import sleep


class LocalCacheObject:
    def __init__(self,name):
        print(f"LocalCacheObject({name}  {id(self)}) constructor at {getpid()}")
        self.name=name
    def call(self,message):
        print(f"Local cache object {self.name}  {id(self)} at {getpid()} got {message}")

class CentralCacheObject:
    def __init__(self,name):
        print(f"CentralCacheObject({name}  {id(self)}) constructor at {getpid()}")
        self.name=name
    def call(self,message):
        print(f"Central cache object {self.name}  {id(self)} at {getpid()} got {message}")


def get_cache(d):
    if "central_cache" in d:
        return d["central_cache"]
    if "local_cache" in d:
        f=d["local_cache"]
        arg=d.get("arg",[])
        if arg is None:
            arg=[]
        return f(*arg)
    return LocalCacheObject("default")

def register_cache_constructor(ns,f):
    print("register_cache_constructor 1",ns,getpid())
    ns.cache_constructor=f
    print("register_cache_constructor 2",ns,getpid())
    return "register_cache_constructor DONE"

class MyManager(BaseManager):
    pass

MyManager.register("CentralCache",CentralCacheObject)

def init(local=True):
    print ("init",getpid())
    manager = MyManager()
    manager.start()    
    print ("init manager=", manager, getpid())

    if local:
        d=dict(local_cache=LocalCacheObject,arg=["LOCAL"])
    else:
        d=dict(central_cache=manager.CentralCache("CENTRAL"))
    print ("init d=",d,getpid())
    return d


def work(d,message):
    print ("work ", d, getpid())
    get_cache(d).call(f"Hello {message} from {getpid()}")
    return f"work DONE {getpid()}"

if __name__=="__main__":
    print("MAIN",getpid())
    p = init(False)
    if True:
        pool = mp.Pool(4)
        pool.apply_async(work,[p,"A"], callback=print)
        pool.apply_async(work,[p,"B"], callback=print)
        pool.close()
        pool.join()
    else:
        a=mp.Process(target=work, args=[p,"AA"])
        b=mp.Process(target=work, args=[p,"BB"])
        a.start()
        b.start()
        a.join()
        b.join()

#    while True:
#        sleep(1)



