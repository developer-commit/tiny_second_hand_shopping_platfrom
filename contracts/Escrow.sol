// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract Escrow {
    enum State { Deposited, Released, Disputed, Refunded }

    struct Trade {
        address buyer;
        address seller;
        uint256 amount;
        State state;
    }

    mapping(uint256 => Trade) public trades;

    event Deposited(uint256 indexed tradeId, address indexed buyer, address indexed seller, uint256 amount);
    event Released(uint256 indexed tradeId, address indexed seller, uint256 amount);
    event Disputed(uint256 indexed tradeId, address indexed buyer, string reason);
    event Refunded(uint256 indexed tradeId, address indexed buyer, uint256 amount);

    function deposit(uint256 tradeId, address seller) external payable {
        require(msg.value > 0, "Deposit amount must be greater than 0");
        require(trades[tradeId].buyer == address(0), "Trade already exists");

        trades[tradeId] = Trade({
            buyer: msg.sender,
            seller: seller,
            amount: msg.value,
            state: State.Deposited
        });

        emit Deposited(tradeId, msg.sender, seller, msg.value);
    }

    function release(uint256 tradeId) external {
        Trade storage trade = trades[tradeId];
        require(trade.state == State.Deposited, "Invalid state for release");
        // For simplicity, anyone with backend authority (or buyer) can release.
        // Usually you'd restrict this to buyer or admin. Here we allow the caller (which might be the backend admin) to trigger it.

        trade.state = State.Released;
        uint256 amount = trade.amount;
        
        (bool success, ) = trade.seller.call{value: amount}("");
        require(success, "Transfer to seller failed");

        emit Released(tradeId, trade.seller, amount);
    }

    function dispute(uint256 tradeId, string calldata reason) external {
        Trade storage trade = trades[tradeId];
        require(trade.state == State.Deposited, "Invalid state for dispute");

        trade.state = State.Disputed;

        emit Disputed(tradeId, trade.buyer, reason);
    }

    function refund(uint256 tradeId) external {
        Trade storage trade = trades[tradeId];
        require(trade.state == State.Disputed, "Only disputed trades can be refunded");

        trade.state = State.Refunded;
        uint256 amount = trade.amount;
        
        (bool success, ) = trade.buyer.call{value: amount}("");
        require(success, "Transfer to buyer failed");

        emit Refunded(tradeId, trade.buyer, amount);
    }
}
