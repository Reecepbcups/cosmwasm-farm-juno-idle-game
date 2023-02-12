from enum import Enum
from time import sleep

"""
Farm based idle game in CosmWasm
Crops, Workers, Animals. Each give a different amount of points per block when sold on the market.
"""


def yield_block():
    for i in range(1, 1_000_000):
        yield i
        sleep(0.05)


class AssetType(Enum):
    CROPS = 1
    WORKERS = 2
    ANIMALS = 3


class Asset:
    # growth_rate_inc, lower is faster
    def __init__(
        self,
        amount: int = 1,
        growth_rate: int = 10_000,
        growth_rate_inc: int = 100,  # growth_rate / growth_rate_inc = increase. (10_000/100 = 100 + 10_000 = 10_100. so 1% faster production)
        cost: int = 1_000_000,
        cost_inc: int = 5,  # (1_000_000/5 = 200_000 + 1_000_000 = 1_200_000. so 20% more expensive to upgrade)
    ):
        self.amount = amount

        self.growth_rate = growth_rate  # per block
        self.growth_rate_inc = growth_rate_inc

        self.cost = cost  # cost to upgrade points
        self.cost_inc = cost_inc

    def upgrade(self, user_points: int) -> int:
        # returns the number of points to remove from user
        current_cost = self.cost
        if user_points < current_cost:
            print("Not enough points")
            return 0

        # +10% growth rate per block
        self.growth_rate = (
            int(self.growth_rate / self.growth_rate_inc) + self.growth_rate
        )
        # +20% cost to upgrade for the next time
        self.cost = int(self.cost / self.cost_inc) + self.cost

        return current_cost

    def __str__(self):
        return f"AssetAttributes(amount={self.amount:,}, growth_rate={self.growth_rate:,}, cost={self.cost:,})"

    def __repr__(self):
        return self.__str__()


class Game:
    # 1 million points = 1 token (redeemable in the future with tokenfactory?)
    def __init__(self, address, current_block: int):
        self.address = address
        self.points = 0
        self.last_claim = int(current_block)
        self.upgrades: dict = {
            AssetType.CROPS: Asset(
                amount=1,
                growth_rate=10_000,
                growth_rate_inc=100,
                cost=1_000_000,
                cost_inc=10,
            ),
            AssetType.ANIMALS: Asset(
                amount=0,
                growth_rate=30_000,
                growth_rate_inc=90,
                cost=10_000_000,
                cost_inc=5,
            ),
            AssetType.WORKERS: Asset(
                amount=0,
                growth_rate=70_000,
                growth_rate_inc=80,
                cost=15_000_000,
                cost_inc=2,
            ),
        }

    def calc_points(self, block_difference: int):
        tmp_points = 0
        for k, v in self.upgrades.items():
            tmp_points += v.amount * v.growth_rate * block_difference
        return tmp_points

    def claim(self, current_block):
        self.points += self.calc_points(current_block - self.last_claim)
        self.last_claim = current_block
        print(f"Claimed {self.points} points.")

    def upgrade(self, asset_type: AssetType):
        if asset_type not in self.upgrades:
            print("Invalid asset type")
            return

        cost = self.upgrades[asset_type].upgrade(self.points)
        self.points -= cost

        if cost == 0:
            print(f"Did not upgrade {asset_type}")
        else:
            print(f"Upgraded {asset_type} for {cost} points.")
            print(self.upgrades[asset_type])

    def get_upgrade_cost(self, asset_type: AssetType):
        if asset_type not in self.upgrades:
            print("Invalid asset type")
            return

        return self.upgrades[asset_type].cost

    def admin_add(self, points: int):
        self.points += points
        print(
            f"ADMIN: Added {points} points to {self.address}. New total: {self.points} points."
        )

    def __str__(self):
        return f"Game(address={self.address}, points={self.points}, last_claim={self.last_claim}, upgrades={self.upgrades})"


def main():
    for b in yield_block():
        print(b)

        if b == 2:
            g = Game("0x123", b)
            print(g)

        if b == 4:
            g.claim(b)
            print(g)

        if b == 10:
            g.claim(b)
            print(g)

        if b == 11:
            print(f"\n\nUpgrade cost: {g.get_upgrade_cost(AssetType.CROPS)}")
            g.admin_add(920_000)
            g.upgrade(AssetType.CROPS)
            print(g)

        if b == 12:
            g.claim(b)

        if b == 15:
            g.claim(b)

        if b == 40:
            g.claim(b)
            exit(1)


if __name__ == "__main__":
    main()
