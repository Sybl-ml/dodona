import React from "react";
import { Card } from 'react-bootstrap';
import styled from "styled-components";

export const TextCard = styled(Card)` 
    background-color: ${({ theme }) => theme.body};
    color: ${({ theme }) => theme.text};
  }
`;


export const ItemCardTemplate = styled(Card)` 
    background-color: ${({ theme }) => theme.body};
    color: ${({ theme }) => theme.text};
    width: 100%;
    margin: 0.5rem;
    padding: 0.5rem;
    text-align: left;
    box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2);
  }
`;



export const ItemCard = ({ children }) => {
  return (
    <ItemCardTemplate>
      <h5>Process #4</h5>
      <h6>Time Elapsed: 6 Days</h6>
    </ItemCardTemplate>
  );
};