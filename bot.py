import os
import sqlite3
from contextlib import closing

import asyncio
import discord
from discord.ext import commands

from dotenv import load_dotenv

con = sqlite3.connect("nuki.db")

with closing(con.cursor()) as cur:
    cur.execute(
        """create table if not exists counter(discord_uid primary key,
                count default 0)"""
    )


class MyBot(commands.Bot):
    def __init__(self) -> None:
        intents = discord.Intents.default()
        intents.message_content = True
        super().__init__(command_prefix="/", intents=intents)


class Nukitally(commands.Cog):
    def __init__(self, bot):
        self.bot = bot

    @commands.command()
    async def nuki(self, ctx, arg: int = 1):
        count = arg
        if count <= 0:
            await ctx.send(f"Please enter a positive number.")
            return

        with closing(con.cursor()) as cur:
            cur.execute(
                """insert into counter(discord_uid, count) values (?, ?) on
                        conflict(discord_uid) do update set count=count+?""",
                (ctx.message.author.id, count, count),
            )

        con.commit()

        await ctx.send(
            f"<@{ctx.message.author.id}> nuki'd {count} time{'s' if count != 1 else ''}!"
        )

    @commands.command()
    async def nukicount(self, ctx, member: discord.Member = None):
        if member is None:
            member = ctx.message.author

        count = 0
        with closing(con.cursor()) as cur:
            res = cur.execute(
                """select count from counter where discord_uid=?""",
                [member.id],
            )

            data = res.fetchone()
            if data is not None:
                count = data[0]

        con.commit()

        await ctx.send(f"<@{member.id}> has {count} nukis.")


async def main():
    bot = MyBot()

    load_dotenv()
    discord.utils.setup_logging()

    await bot.add_cog(Nukitally(bot))
    await bot.start(os.getenv("DISCORD_TOKEN"))


if __name__ == "__main__":
    import asyncio

    asyncio.run(main())
