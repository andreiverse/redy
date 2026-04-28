import asyncio
import nats
from nats.js.errors import BadRequestError, Error
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
                
                # Use run_in_executor to avoid blocking the event loop with synchronous DB/ML work
                loop = asyncio.get_event_loop()
                status = await loop.run_in_executor(
                    None, 
                    sentimental_analysis_worker.handle_sentimental_analysis, 
                    raw_uuid
                )

                if (status == utils.Status.SUCCESS) or (status == utils.Status.INVALID):
                    await msg.ack()
                else:
                    print(f"Task failed for {raw_uuid}, sending nak with delay")
                    await msg.nak(delay=config.TIMEOUT_ON_FAIL_SECONDS)
            else:
                print(f"Received non-UUID data (length {len(msg.data)}): {msg.data}")
                await msg.ack()
        
        except Exception as e:
            print(f"Error processing message: {e}")
            try:
                await msg.nak(delay=config.TIMEOUT_ON_FAIL_SECONDS)
            except Exception:
                pass

    durable_name = "sentimental-analysis-worker"
    queue_group = "sentimental-analysis-worker"

    try:
        await js.subscribe(
            subject, 
            cb=message_handler, 
            durable=durable_name,
            queue=queue_group,
            manual_ack=True
        )
    except Error as e:
        if "cannot create a queue subscription for a consumer without a deliver group" in str(e):
            print(f"Consumer '{durable_name}' exists but is not configured as a queue group. Recreating...")
            try:
                await js.delete_consumer(stream_name, durable_name)
            except Exception as delete_e:
                print(f"Error deleting consumer: {delete_e}")
            
            await js.subscribe(
                subject, 
                cb=message_handler, 
                durable=durable_name,
                queue=queue_group,
                manual_ack=True
            )
        else:
            raise e
    
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