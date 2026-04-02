// SPDX-License-Identifier: MIT
pragma solidity 0.8.20;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

/// @notice P2P lending contract — one instance per loan.
/// @dev Deployed by the LendChain backend on behalf of the borrower.
contract LendChain is ReentrancyGuard {

    // ── State ──────────────────────────────────────────────────────────────
    IERC20  public immutable usdc;
    address public immutable borrower;
    address public immutable lender;     // set at construction or fundLoan
    uint256 public immutable amountUsdc; // principal in USDC (6 decimals)
    uint256 public immutable termMonths;
    uint256 public immutable annualRateBps; // e.g. 500 = 5.00%

    enum Status { PENDING, FUNDED, REPAID, DEFAULTED }
    Status  public status;

    uint256 public totalRepaid;
    uint256 public fundedAt;

    // ── Events ────────────────────────────────────────────────────────────
    event LoanCreated(address indexed borrower, uint256 amount, uint256 termMonths);
    event LoanFunded(address indexed lender, uint256 amount, uint256 fundedAt);
    event PaymentMade(address indexed borrower, uint256 amount, uint256 totalRepaid);
    event LoanClosed(uint256 closedAt);

    // ── Custom errors ─────────────────────────────────────────────────────
    error NotLender();
    error NotBorrower();
    error LoanNotPending();
    error LoanNotFunded();
    error LoanAlreadyClosed();
    error InsufficientAllowance();
    error TransferFailed();

    constructor(
        address _usdc,
        address _borrower,
        uint256 _amountUsdc,
        uint256 _termMonths,
        uint256 _annualRateBps,
        address _lender
    ) {
        usdc           = IERC20(_usdc);
        borrower       = _borrower;
        amountUsdc     = _amountUsdc;
        termMonths     = _termMonths;
        annualRateBps  = _annualRateBps;
        lender         = _lender;
        status         = Status.PENDING;
        emit LoanCreated(_borrower, _amountUsdc, _termMonths);
    }

    /// @notice Lender calls this to transfer USDC to the borrower.
    function fundLoan() external nonReentrant {
        if (msg.sender != lender)        revert NotLender();
        if (status != Status.PENDING)    revert LoanNotPending();
        if (usdc.allowance(msg.sender, address(this)) < amountUsdc)
            revert InsufficientAllowance();

        status   = Status.FUNDED;
        fundedAt = block.timestamp;

        bool ok = usdc.transferFrom(msg.sender, borrower, amountUsdc);
        if (!ok) revert TransferFailed();

        emit LoanFunded(msg.sender, amountUsdc, fundedAt);
    }

    /// @notice Borrower calls this to pay a monthly installment.
    /// @param amount USDC amount for this payment (6 decimals).
    function makePayment(uint256 amount) external nonReentrant {
        if (msg.sender != borrower)   revert NotBorrower();
        if (status != Status.FUNDED)  revert LoanNotFunded();

        if (usdc.allowance(msg.sender, address(this)) < amount)
            revert InsufficientAllowance();

        bool ok = usdc.transferFrom(msg.sender, lender, amount);
        if (!ok) revert TransferFailed();

        totalRepaid += amount;
        emit PaymentMade(msg.sender, amount, totalRepaid);

        // Auto-close when fully repaid (simple check — backend validates exact schedule)
        uint256 totalDue = amountUsdc + (amountUsdc * annualRateBps * termMonths) / (12 * 10000);
        if (totalRepaid >= totalDue) {
            status = Status.REPAID;
            emit LoanClosed(block.timestamp);
        }
    }

    /// @notice Returns current loan state for the backend.
    function getState() external view returns (
        uint8   _status,
        uint256 _totalRepaid,
        uint256 _fundedAt,
        uint256 _amountUsdc
    ) {
        return (uint8(status), totalRepaid, fundedAt, amountUsdc);
    }
}
