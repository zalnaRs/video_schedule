import asyncio
import json
import time
from datetime import datetime
import os
from python_mpv_jsonipc import MPV

# Schedule task structure
class VideoTask:
    def __init__(self, delay, file_path, start_time):
        self.delay = delay
        self.file_path = file_path
        self.start_time = start_time

# Parse video tasks from the schedule.json file
async def parse_schedule():
    with open('schedule.json', 'r') as f:
        data = json.load(f)
    schedule = []
    for task in data:
        schedule.append(VideoTask(task['delay'], task['file_path'], task['start_time']))
    return schedule

nexttask = False


# Wrapper to move to the next task when video ends


# Main function to process tasks
async def main():
    # Create an MPV instance
    player = MPV(mpv_location="./mpv_dir/mpv.exe")

    # Wait for the player to be ready
    # await player.wait_for_ready()

    schedule = await parse_schedule()

    for i in range(0, len(schedule)):
        delay = schedule[i].delay
        if delay == "end":
            nexttask = True
        else:
            nexttask = False
            delay_time = datetime.strptime(delay, "%H:%M:%S")
            total_delay_seconds = delay_time.hour * 3600 + delay_time.minute * 60 + delay_time.second

            if total_delay_seconds > 0:
                await asyncio.sleep(total_delay_seconds)

        print(f"Playing: {schedule[i].file_path} at {schedule[i].start_time} seconds")

        # Load the file
        player.loadfile(schedule[i].file_path)
        #await asyncio.sleep(2)
        #player.seek("60")

        # Seek to the start time
        #player.bind_event("file-loaded", lambda arg: seek(player, schedule[i].start_time))
        @player.on_event("file-loaded")
        def seek(arg):
            player.seek(schedule[i].start_time, "absolute")

        # Bind the end_file event to go to the next task when the video ends
        if delay == "end":
            @player.on_event("end-file")
            def handle_end_file(arg):
                if i + 1 < len(schedule):
                    print(f"Moving to next task: {schedule[i + 1].file_path}")
                    player.loadfile(schedule[i + 1].file_path)

        # Wait 1 second before processing the next task
        await asyncio.sleep(1)

# Run the main function
if __name__ == "__main__":
    asyncio.run(main())
