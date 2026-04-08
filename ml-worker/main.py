import asyncio
import nats
from nats.js.errors import BadRequestError
from nats.js.api import StreamConfig
import uuid
import config
import repository

import sentimental_analysis_worker

from config import NATS_SERVER
import utils

async def run():
    nc = await nats.connect(NATS_SERVER)
    js = nc.jetstream()

    stream_name = "ML"
    subject = "tasks.ml.sentimental-analysis"

    try:
        await js.add_stream(
            name=stream_name, 
            subjects=[subject]
        )
        print(f"Stream '{stream_name}' confirmed.")
    except BadRequestError:
        print(f"Stream '{stream_name}' already exists.")

    async def message_handler(msg):
        try:
            if len(msg.data) == 16:
                raw_uuid = uuid.UUID(bytes=msg.data)
                
                status = sentimental_analysis_worker.handle_sentimental_analysis(raw_uuid)

                if (status == utils.Status.SUCCESS) or (status == utils.Status.INVALID):
                    await msg.ack()
                else:
                    print("sending nak with delay")
                    await msg.nak(config.TIMEOUT_ON_FAIL_SECONDS)
            else:
                print(f"Received non-UUID data (length {len(msg.data)}): {msg.data}")
        
        except Exception as e:
            print(f"Could not parse message: {e}")

    await js.subscribe(
        subject, 
        cb=message_handler, 
        durable="sentimental-analysis-worker"
    )
    
    print(f"Listening on {subject}...")

    try:
        while True:
            await asyncio.sleep(1)
    except KeyboardInterrupt:
        print("\nShutting down...")
    finally:
        await nc.close()

if __name__ == '__main__':
    try:
        asyncio.run(run())
    except KeyboardInterrupt:
        pass