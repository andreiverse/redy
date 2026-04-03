import asyncio
import nats
from nats.js.errors import BadRequestError
from nats.js.api import StreamConfig
import uuid

async def run():
    nc = await nats.connect("nats://localhost:4222")
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
                print(f"Received UUID object: {raw_uuid}")
            else:
                print(f"Received non-UUID data (length {len(msg.data)}): {msg.data}")
        
        except Exception as e:
            print(f"Could not parse message: {e}")
        await msg.nak(30)

    await js.subscribe(
        subject, 
        cb=message_handler, 
        durable="sentiment-debugger"
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