import {cycleAllElements} from './cycleAnimation'
import { dataTable } from './dataTable';

function init(): void {
  cycleAllElements('.js-cycle');
  dataTable('#data', '[data-js="data-table"]')
}

init();