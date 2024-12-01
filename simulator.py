import numpy as np
from tqdm import tqdm
import pandas as pd
import time
import multiprocessing
import subprocess
import json
import datetime


def run(i):
    output_str = subprocess.run(f'powershell cat in/{i:04}.txt | .\\target\\debug\\ahc040.exe > out/{i:04}.txt', shell=True, capture_output=True, text=True).stderr
    # print('output_str:', output_str.split('\n'))
    result = json.loads(output_str.split('\n')[-2])
    return result


def main(i):
    start = time.time()
    # print(i, 'start')
    r = run(i)
    t = round(time.time()-start, 4)
    N = r['N']
    T = r['T']
    S = r['S']
    estimated_score = r['estimated score']
    score = r['score']
    real_score = r['real score']
    data = [i, N, T, S, estimated_score, score, real_score, t]
    print('\r', 'end', i, end='')
    # print(i, 'end')
    return data


if __name__ == '__main__':
    start = time.time()
    print("start: ", datetime.datetime.fromtimestamp(start))
    trial = 50
    '''
    result = []
    for i in tqdm(range(trial)):
        data = main(i)
        result.append(data)
    '''
    processes = multiprocessing.cpu_count()
    with multiprocessing.Pool(processes=processes) as pool:
        data = [pool.apply_async(main, (i,)) for i in range(trial)]
        result = [d.get() for d in data]
    print()
    # '''
    df = pd.DataFrame(result, columns=['i', 'N', 'T', 'S', 'estimated score', 'score', 'real score', 'time'])
    estimated_score = np.mean(df['estimated score'])
    score = np.mean(df['score'])
    real_score = np.mean(df['real score'])
    sum_score = score * 50
    sum_estimated_score = estimated_score * 50
    sum_real_score = real_score * 50
    print(f"judge score: {format(int(sum_score), ',')}, score mean: {format(int(score), ',')}")
    print(f"estimated_score: {format(int(sum_estimated_score), ',')}, score mean: {format(int(estimated_score), ',')}")
    print(f"real_score: {format(int(sum_real_score), ',')}, score mean: {format(int(real_score), ',')}")
    df.to_csv('result.csv', index=False)
    print(f'end elapsed time: {time.time()-start:.2f}s')
