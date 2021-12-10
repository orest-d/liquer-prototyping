import pyarrow
import datafusion as daf
import pandas as pd
from liquer import *
import liquer.ext.lq_pandas
import numpy as np
import pyarrow.parquet as pq

@first_command
def harmonic(n=100):
    a = np.linspace(0,2*np.pi,n)
    segment = np.array(a*10/(2*np.pi),dtype=int)
    return pd.DataFrame(
        dict(
            a=a,
            x=np.sin(a),
            y=np.cos(a),
            segment=segment,
            label=[f"{i+1}/{n}" for i in range(n)]
        )
    )

@first_command
def harmonic2(n=100):
    a = np.linspace(0,2*np.pi,n)
    segment = np.array(a*10/(2*np.pi),dtype=int)
    return pd.DataFrame(
        dict(
            a=a,
            x2=np.sin(2*a),
            y2=np.cos(2*a),
            segment=segment,
            label=[f"{i+1}/{n}" for i in range(n)]
        )
    )

evaluate_and_save("harmonic/harmonic.parquet",".")
evaluate_and_save("harmonic2/harmonic2.parquet",".")

ctx = daf.ExecutionContext()
ctx.register_parquet("a","harmonic.parquet")
ctx.register_parquet("b","harmonic2.parquet")
df=ctx.sql("""
SELECT * FROM a WHERE a>1
""")
print(df.show())

table = pyarrow.Table.from_batches(df.collect())
pq.write_table(table, 'result.parquet')

df = pd.read_parquet("result.parquet")
print(df)

df=ctx.sql("""
SELECT
  a.a as a,
  a.x as x,
  a.y as y,
  b.x2 as xx,
  b.y2 as yy
FROM a,b WHERE a.a=b.a
""")
print(df.show())
