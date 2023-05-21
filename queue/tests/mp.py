import multiprocessing as mp
from multiprocessing.managers import BaseManager, SyncManager, Namespace
from os import getpid
from time import sleep


class LocalCacheObject:
    def __init__(self,name):
        self.name=name
    def call(self,message):
        print(f"Local cache object {self.name} at {getpid()} got {message}")

class CentralCacheObject:
    def __init__(self,name):
        self.name=name
    def call(self,message):
        print(f"Central cache object {self.name} at {getpid()} got {message}")

class Foo:
    def f(self):
        print('you called Foo.f()')
    def g(self):
        print('you called Foo.g()')
    def _h(self):
        print('you called Foo._h()')

def register_cache_constructor(ns,f):
    print("register_cache_constructor 1",ns,getpid())
    ns.cache_constructor=f
    print("register_cache_constructor 2",ns,getpid())
    return "register_cache_constructor DONE"

class MyManager(BaseManager):
    pass

MyManager.register("Foo1",Foo)
MyManager.register("CentralCache",CentralCacheObject)

def init():
    print ("init",getpid())
    manager = mp.Manager()
#    manager = MyManager()
#    manager.start()    
    print ("init manager=", manager, getpid())
    ns = manager.Namespace()
    print ("init ns1=",ns,getpid())
    ns.cache_constructor=None
    ns.local_cache_constructor=LocalCacheObject
    ns.local_cache_parameter="A"
    f=manager.Foo1()
    f.f()
    central_cache=manager.CentralCache("B")
    ns.central_cache=central_cache
    print ("init ns2=",ns,getpid())
    return ns

def constructor():
    print("constructor",getpid())
    return "constructor DONE"

def call_cache_constructor(ns):
    print ("call_cache_constructor ns=", ns, getpid())
    while ns.cache_constructor is None:
        print("  ping cache constructor", ns, getpid())
    print("  got cache constructor", ns, getpid())
    print ("CALL cache_constructor", getpid())
    print(ns.cache_constructor())
    print ("DONE", getpid())
    return "call_cache_constructor DONE"

if __name__=="__main__":
    print("MAIN",getpid())
    ns = init()
    if True:
        pool = mp.Pool(4)
        pool.apply_async(call_cache_constructor,[ns], callback=print)
        pool.apply_async(register_cache_constructor,[ns,constructor], callback=print)
        pool.close()
        pool.join()
    else:
        a=mp.Process(target=call_cache_constructor, args=[ns])
        b=mp.Process(target=register_cache_constructor, args=[ns,constructor])
        a.start()
        b.start()
        a.join()
        b.join()

#    while True:
#        sleep(1)



