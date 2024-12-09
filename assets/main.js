const select = document.querySelector('.select');
const selected = document.querySelector('.selected');


document.addEventListener('DOMContentLoaded', () => {
    console.log('DOM полностью загружен');
    // Добавляем функциональность открытия/закрытия списка
    document.querySelector('.select').addEventListener('click', function(event) {
        const options = this.querySelector('.options');
        options.classList.toggle('show');
        selected.classList.toggle('open');
    });

    // Закрываем выпадающий список при клике вне его
    document.addEventListener('click', function(event) {

        if (!select.contains(event.target)) {
            select.querySelector('.options').classList.remove('show');
            selected.classList.remove('open');
        }
    });
});