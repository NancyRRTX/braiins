# Overview

This project is a simulation of the Stratum mining protocol. It currently supports both versions: **Stratum V1** and **Stratum V2**. The intention is to verify the design of **Stratum V2** with regards to translating between both protocol variants. At the same time, the platform can serve as a testbed for various network latency scenarios.

Last but not least, the idea is to have a reference implementation of both protocols that serves the blueprint specification of the messages.


# Features

- test network latency issues
- complete definition of protocol messages
- pool rejects stale shares since it simulates finding new blocks
- miners simulate finding shares based exponential distribution
- plot results of series of simmulations


## Install

Simulation requires Python 3.7.

The easiest way to run the simulation is to use python `virtualenvwrapper`


### The `virtualenvwrapper` way

```
mkvirtualenv --python=/usr/bin/python3.7 stratum-sim
pip install -r ./requirements.txt
```

## Running Stratum V2 Simulation

`python ./pool_miner_sim.py --verbose --latency=0.2`

## Running Stratum V1 Simulation

`python ./pool_miner_sim.py --verbose --latency=0.2 --v1`

## Running Stratum V2 Miner and V1 Pool using Proxy Simulation

`python ./pool_miner_sim.py --verbose --latency=0.2 --v2v1`

## Simulate V2-V2, V1-V1 and V2-proxy-V1 and plot results into PDF report

`python ./simulate_and_plot_results.py`


# Future Work

The simulation is far from complete. Currently, it supports the following
 scenarios:

```
2xminer (V1) ----> pool (V1)
2xminer (V2) ----> pool (V2)
miner (V2) ----> proxy (translating) ---> pool (V1)
```

Example scenarios that need to be to be covered:

```
miner (V1) ----> proxy (V1:V2) ---> pool (V2)
```

The current simulation output is very basic, and we know that it could be extended much further. Below are a few points that could be covered in future iterations:
- implement BDD scenarios using gherkin language to run a full set of simulation scenarios
- provide more advanced statistics with chart plotting
