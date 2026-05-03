import asyncio
import nats
from nats.js.errors import BadRequestError
import uuid
import json

import sentimental_analysis_worker
import categorize_worker

from config import NATS_SERVER
import config
import utils


STREAM_NAME = "ML"
SUBJECTS = ["tasks.ml.*"]
DURABLE_NAME = "ml-worker"
QUEUE_GROUP = "ml-worker"


async def ensure_stream(js):
    try:
        info = await js.stream_info(STREAM_NAME)

        current_subjects = set(info.config.subjects)
        expected_subjects = set(SUBJECTS)

        if current_subjects != expected_subjects:
            print("Stream subjects mismatch. Updating stream...")

            await js.update_stream(
                name=STREAM_NAME,
                subjects=SUBJECTS
            )
        else:
            print("Stream is valid.")

    except Exception:
        print("Creating stream...")
        await js.add_stream(
            name=STREAM_NAME,
            subjects=SUBJECTS
        )


async def run():
    nc = await nats.connect(NATS_SERVER)
    js = nc.jetstream()

    await ensure_stream(js)

    async def message_handler(msg):
        try:
            payload = json.loads(msg.data.decode())
            article_uuid_str = payload.get("article_uuid")
            if article_uuid_str is None:
                print(f"Task {msg.subject} received without article_uuid, skipping.")
                await msg.ack()
                return

            article_uuid = uuid.UUID(article_uuid_str)
            subject = msg.subject

            loop = asyncio.get_event_loop()

            if subject == "tasks.ml.sentimental-analysis":
                status = await loop.run_in_executor(
                    None,
                    sentimental_analysis_worker.handle_sentimental_analysis,
                    article_uuid
                )

            elif subject == "tasks.ml.categorize":
                status = await loop.run_in_executor(
                    None,
                    categorize_worker.handle_categorize,
                    article_uuid
                )

            else:
                print(f"Unknown subject: {subject}")
                await msg.ack()
                return

            if status in (utils.Status.SUCCESS, utils.Status.INVALID):
                await msg.ack()
            else:
                await msg.nak(delay=config.TIMEOUT_ON_FAIL_SECONDS)

        except Exception as e:
            print(f"Error: {e}")
            try:
                await msg.nak(delay=config.TIMEOUT_ON_FAIL_SECONDS)
            except:
                pass

    try:
        await js.subscribe(
            "tasks.ml.*",
            cb=message_handler,
            durable=DURABLE_NAME,
            queue=QUEUE_GROUP,
            manual_ack=True
        )
    except Exception as e:
        print("Consumer mismatch. Recreating...")

        try:
            await js.delete_consumer(STREAM_NAME, DURABLE_NAME)
        except:
            pass

        await js.subscribe(
            "tasks.ml.*",
            cb=message_handler,
            durable=DURABLE_NAME,
            queue=QUEUE_GROUP,
            manual_ack=True
        )

    print("Listening on tasks.ml.* ...")

    while True:
        await asyncio.sleep(1)


if __name__ == "__main__":
    asyncio.run(run())
