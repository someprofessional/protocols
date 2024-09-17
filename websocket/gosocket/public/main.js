document.addEventListener("DOMContentLoaded", () => {
    const socket = new WebSocket("ws://localhost:8000/socketme");

    socket.onopen = () => {
        console.log("WebSocket connection established");
    }

    socket.onmessage = (event) => {
        console.log("Data received from server : ", event.data);

        const rankingData = JSON.parse(event.data);

        updateTable(rankingData);
    }

    socket.onerror = (error) => {
        console.error("Web socket error : ", error);
    }

    socket.onclose = () => {
        console.log("connection closed")
    }

    function updateTable(data) {
        const tableBody = document.querySelector("#rankingTable tbody");

        tableBody.innerHTML = '';

        data.forEach((entry, index) => {
            const row = document.createElement("tr");

            const rankCell = document.createElement("td");
			rankCell.textContent = index + 1;
			row.appendChild(rankCell);

			const nameCell = document.createElement("td");
			nameCell.textContent = entry.name;
			row.appendChild(nameCell);

			const scoreCell = document.createElement("td");
			scoreCell.textContent = entry.score;
			row.appendChild(scoreCell);

			tableBody.appendChild(row);
        });
    }
})
