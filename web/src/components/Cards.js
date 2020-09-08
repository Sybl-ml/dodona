import { Card } from 'react-bootstrap';
import styled from "styled-components";

export const TextCard = styled(Card)` 
    background-color: ${({ theme }) => theme.body};
    color: ${({ theme }) => theme.text};
  }
`;
