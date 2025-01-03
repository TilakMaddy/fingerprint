// SPDX-License-Identifier: MIT
pragma solidity 0.8.19;

library ChatUtils {
    function deriveKey(
        address person1,
        address person2
    ) internal pure returns (bytes32) {
        require(person1 != address(0));
        require(person2 != address(0));
        require(person1 != person2, "you cannot make friends with self");
        address f1;
        address f2;
        if (person1 < person2) {
            f1 = person1;
            f2 = person2;
        } else {
            f1 = person2;
            f2 = person1;
        }
        return keccak256(abi.encode(f1, f2));
    }
}

contract Chat {
    enum Status {
        NotConsidered, // default
        Pending,
        Accepted,
        Rejected
    }

    struct Message {
        address who; // Doesn't matter what you fill. The function will force set it to msg.sender
        bytes nonce;
        bytes encryptedMessage;
        uint256 when; // Doesn't matter what this is set to. Same as above
        bool isFile;
    }

    struct Friendship {
        Status status;
        mapping(address => bytes) dhPubKeys;
        Message[] messages;
    }

    mapping(bytes32 => Friendship) public s_friendship;
    mapping(address => address[]) public s_finalFriends;
    mapping(address => address[]) public s_pendingRequests;
    mapping(address => address[]) public s_allSentRequests;

    function getFriendship(
        address f1,
        address f2
    ) internal view returns (Friendship storage) {
        bytes32 friendshipKey = ChatUtils.deriveKey(f1, f2);
        return s_friendship[friendshipKey];
    }

    // State mutating Changes

    function addFriend(
        address person,
        bytes memory dhPubKey
    ) external returns (bool) {
        Friendship storage f = getFriendship(person, msg.sender);
        if (f.status == Status.NotConsidered) {
            f.status = Status.Pending;
            f.dhPubKeys[msg.sender] = dhPubKey;
            s_pendingRequests[person].push(msg.sender);
            s_allSentRequests[msg.sender].push(person);
            return false;
        } else if (f.status == Status.Pending) {
            f.status = Status.Accepted;
            f.dhPubKeys[msg.sender] = dhPubKey;
            s_finalFriends[msg.sender].push(person);
            s_finalFriends[person].push(msg.sender);
            address[] storage pending = s_pendingRequests[msg.sender];
            bool found = false;
            // This is O(n)
            for (uint256 i = 0; i < pending.length; ++i) {
                if (pending[i] == person) {
                    // Leaves a gap
                    delete pending[i];
                    for (uint256 j = i + 1; j < pending.length; ++j) {
                        pending[j - 1] = pending[j];
                    }
                    found = true;
                    break;
                }
            }
            require(found, "not allowed");
            return true;
        }
        revert();
    }

    function rejectFriendship(address person) external {
        Friendship storage f = getFriendship(person, msg.sender);
        require(
            f.status == Status.Pending,
            "you can only reject at pending stage"
        );
        address[] storage pending = s_pendingRequests[msg.sender];
        // This is O(n)
        bool found = false;
        for (uint256 i = 0; i < pending.length; ++i) {
            if (pending[i] == person) {
                // Leaves a gap
                delete pending[i];
                for (uint256 j = i + 1; j < pending.length; ++j) {
                    pending[j - 1] = pending[j];
                }
                found = true;
                break;
            }
        }
        require(found, "nooooo");
        f.status = Status.Rejected;
    }

    function sendMessage(address friend, Message memory message) public {
        Friendship storage f = getFriendship(friend, msg.sender);
        require(f.status == Status.Accepted, "you are not friends");
        message.who = msg.sender;
        message.when = block.timestamp;
        f.messages.push(message);
    }

    /// Public view functions

    function getDHPublicKeys(
        address f1,
        address f2
    ) public view returns (bool ok, bytes memory f1Key, bytes memory f2Key) {
        Friendship storage f = getFriendship(f1, f2);
        ok = f.status == Status.Accepted ? true : false;
        f1Key = f.dhPubKeys[f1];
        f2Key = f.dhPubKeys[f2];
    }

    function getFriendshipStatus(
        address f1,
        address f2
    ) public view returns (Status) {
        Friendship storage f = getFriendship(f1, f2);
        return f.status;
    }

    ///////////////////////////////////////////////////

    function getLatestMessages(
        address f1,
        address f2,
        uint256 offset,
        uint256 len
    ) public view returns (Message[] memory) {
        Friendship storage f = getFriendship(f1, f2);
        require(f.status == Status.Accepted, "not friends");
        uint256 N = f.messages.length;

        // Show all messages when len = 0
        if (len == 0) len = N;

        Message[] memory messages = new Message[](len);
        for (uint256 i = offset; i < len; i++) {
            if (N < i + 1) {
                continue;
            }
            uint messageIndex = N - 1 - i;
            messages[i] = f.messages[messageIndex];
        }
        return messages;
    }

    function getLatestMessagesForMe(
        address f2,
        uint256 offset,
        uint256 len
    ) public view returns (Message[] memory) {
        return getLatestMessages(msg.sender, f2, offset, len);
    }

    ///////////////////////////////////////////////////

    function getFriendList(
        address person
    ) public view returns (address[] memory) {
        return s_finalFriends[person];
    }

    function getFriendListForMe() public view returns (address[] memory) {
        return s_finalFriends[msg.sender];
    }

    //////////////////////////

    function getIncomingPendingRequests(
        address person
    ) public view returns (address[] memory) {
        return s_pendingRequests[person];
    }

    function getIncomingRequestsForMe() public view returns (address[] memory) {
        return s_pendingRequests[msg.sender];
    }

    //////////////////////////

    function getOutgoingRequests(
        address person
    ) public view returns (address[] memory, Status[] memory) {
        address[] storage outgoing = s_allSentRequests[person];
        uint256 total = outgoing.length;
        Status[] memory statuses = new Status[](total);
        for (uint256 i = 0; i < total; ++i) {
            address otherPerson = outgoing[i];
            Friendship storage f = getFriendship(person, otherPerson);
            statuses[i] = f.status;
        }
        return (outgoing, statuses);
    }

    function getOutgoingRequestsForMe()
        public
        view
        returns (address[] memory, Status[] memory)
    {
        return getOutgoingRequests(msg.sender);
    }

    function whoAmI() public view returns (address) {
        return msg.sender;
    }
}
