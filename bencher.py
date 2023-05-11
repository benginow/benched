import os 
import time
import re

# pass in test names as command line arguments
# "figure5",
test_names = ["test", "async_match_deadlock", "find_deadlock_config", "minimal_deadlock", "figure5"]
            #   "yield_spin_loop_true", "yield_spin_loop_false", "figure5"]
            #   , "yield_spin_loop_true", "yield_spin_loop_false"]
            # "async_match_deadlock" 
        # "find_deadlock_config"
        # "minimal_deadlock" 
modes = ["RANDOM", "PCT"]
# these have to be rebuilt in order to run, so the bencher works on them separately -- two runs needed!
non_fuzz_modes = []
fuzz_modes = ["FUZZ_W", "FUZZ_RR", "FUZZ_RA", "FUZZ_A", "FUZZ_PCT"]
directory = 'out/default/crashes'

def parse_pct_fuzz(writer, time):
    # now, go through output log

    with open("output.log", "r") as f:
        num_run_out = 0
        num_execs = 0
        # last_schedule_run_out = -1
        current_schedule = -1

        for line in f:
            index = line.find("round ")
            end_index = line.find(",")
            if (index >= 0):
                current_schedule = int(line[index+6:end_index])
                num_execs = num_execs + 1

            # if line.find("we have run out of schedule steps") >= 0:
            #     num_run_out = num_run_out + 1
            #     last_schedule_run_out = current_schedule
                
        print(str(num_run_out) + "\t")
        print(str(num_execs) + "\t")
        print(str(num_run_out) + "\t")
        print(str(time) + "\t")
        
        writer.write(str(num_run_out) + "\t")
        writer.write(str(num_execs) + "\t")
        writer.write(str(float(num_execs - num_run_out) / float(num_execs)) + "\t")
        writer.write(str(time) + "\t")

def parse_party(writer, time): 
    time_till_first_crash_minutes = -1.0
    with open("out/default/plot_data") as fi:
        for line in fi:
            if (line.find("#") >= 0):
                continue
            result = [x.strip() for x in line.split(',')]

            if (int(result[7]) > 0):
                time_till_first_crash_minutes = float(result[0])/60.0
                break

    # now, go through output log

    with open("output.log", "r") as f:
        num_run_out = 0
        num_execs = 0
        last_schedule_run_out = -1
        current_schedule = -1

        for line in f:
            index = line.find("execution{i=")
            end_index = line.find("}")
            if (index >= 0):
                current_schedule = int(line[index+12:end_index])

            if line.find("we have run out of schedule steps") >= 0:
                if (current_schedule != last_schedule_run_out):
                    num_run_out = num_run_out + 1
                    last_schedule_run_out = current_schedule
            i = line.find("new execution")
            
            if i >= 0:
                num_execs = num_execs + 1

        print(str(num_run_out) + "\t")
        print(str(num_execs) + "\t")
        print(str(num_run_out) + "\t")
        print(str(time) + "\t")
        
        writer.write(str(num_run_out) + "\t")
        writer.write(str(num_execs) + "\t")
        writer.write(str(float(num_execs - num_run_out) / float(num_execs)) + "\t")
        writer.write(str(time) + "\t")
        # writer.write(str(time_till_first_crash_minutes) + "\t")

def parse_non_fuzz_party(writer, time):
    # with open("output.log") as f:
    #     # look for number of executions on last line
    #     for line in f.readlines():
    #         begin = line.find("{execution{i=")
    #         if begin >= 0:
    #             numbers = re.findall('[0-9]+', line[begin:])
    #             if (len(numbers > 0)):
    #                 writer.write(numbers[0] + "\t")
    #             else:
    #                 writer.write("-1\t")
    writer.write(str(time) + "\t")


        

f = open("parsed_data.txt", "a")
f.write("\ntestname\t\tschedule steps\t\titerations till crash\t\tvalid schedule fraction\t\tminutes till first crash\n")

non_fuzz_f = open("non_fuzzed_data.txt", "a")
non_fuzz_f.write("\ntestname\t\titerations till crash\t\ttime till crash\n")

clear = "rm output.log\nrm -rf out"

# go through, run each, pase output.log for: NUMBER OF ITERATIONS, NUMBER OF SCHEDULES, 
# TODO: PARSE SCHEDULE CHANGES

for test in test_names:
    for mode in modes:
        
        if mode == "PCT" or mode == "RANDOM" or mode == "ROUND_ROBIN":
            non_fuzz_f.write("\n")
            non_fuzz_f.write(test + " " + mode)
            # if pct or random, need to time the program see how many interleavings are explored as well?
            # cargo run --bin fuzz_target -- --test test --mode mode
            runner = "env RUST_LOG=shuttle=info cargo run --bin non_fuzz_target -- --test " + test + " --mode " + mode
        else:
            f.write("\n")
            f.write(test + " " + mode + " \t")
            runner = "env RUST_LOG=shuttle=info AFL_BENCH_UNTIL_CRASH=1 cargo afl fuzz -i in -o out target/debug/fuzz_target -- --test " + test + " --mode " + mode

        os.system(clear)
        print("running " + runner + " \n")
        start = time.time()
        os.system(runner)
        end = time.time()

        if mode == "PCT" or mode == "RANDOM" or mode == "ROUND_ROBIN":
            parse_non_fuzz_party(non_fuzz_f, end-start)
        elif mode == "FUZZ_PCT":
            parse_pct_fuzz(f, end-start)
        else:
            parse_party(f, end-start)