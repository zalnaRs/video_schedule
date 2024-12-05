import asyncio
import json
from datetime import datetime
from python_mpv_jsonipc import MPV

class VideoTask:
    def __init__(self, delay, file_path, start_time):
        self.delay = delay
        self.file_path = file_path
        self.start_time = start_time

async def parse_schedule():
    with open('schedule.json', 'r') as f:
        data = json.load(f)
    schedule = []
    for task in data:
        schedule.append(VideoTask(task['delay'], task['file_path'], task['start_time']))
    return schedule

async def main():
    player = MPV(mpv_location="./mpv_dir/mpv.exe")
    schedule = await parse_schedule()

    for task in schedule[:]:
        delay_time = datetime.strptime(task.delay, "%H:%M:%S")
        total_delay_seconds = delay_time.hour * 3600 + delay_time.minute * 60 + delay_time.second

        if total_delay_seconds > 0:
            await asyncio.sleep(total_delay_seconds)

        print(f"Playing: {task.file_path} at {task.start_time} seconds")

        player.loadfile(task.file_path)

        @player.on_event("file-loaded")
        def seek(arg):
            player.seek(task.start_time, "absolute")

        await asyncio.sleep(1)

if __name__ == "__main__":
    asyncio.run(main())